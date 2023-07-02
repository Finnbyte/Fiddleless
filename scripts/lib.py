from bs4 import BeautifulSoup
import re
import os
import json as JSON
import urllib.request
import urllib3
from multiprocessing import Pool
from vars import roles


class APIRunes:
    def __init__(self, payload) -> None:
        self._data = payload["pageProps"]["data"]["meta"]
        self._runeCollection = self._data["runes"] + self._data["runePages"] + self._data["statMods"]

    def name_to_id(self, name: str):
        for entry in self._runeCollection:
            if entry["name"] == name:
                return str(entry["id"])

    def id_to_name(self, id: str):
        for entry in self._runeCollection:
            if entry["id"] == int(id):
                return entry["name"]

    def id_to_name_and_id(self, id: str):
        for entry in self._runeCollection:
            if entry["id"] == int(id):
                return (entry["name"], str(id))


class APIChampion:
    def __init__(self, payload) -> None:
        self._data = payload["pageProps"]
        self._runes = APIRunes(payload)

    @property
    def info(self):
        return {
            "name": self._data["champion"].capitalize(),
            "role": self._data["position"]
        }

    @property
    def current_patch(self):
        return {
            "version": self._data["data"]["summary"]["version"]["version"],
            "compatible_versions": self._data["data"]["summary"]["versions"]
        }

    @property
    def optimal_runes(self):
        rune_paths = (
            self._runes.id_to_name_and_id(self._data["data"]["runes"][0]["primary_page_id"]),
            self._runes.id_to_name_and_id(self._data["data"]["runes"][0]["secondary_page_id"]))
        keystone = self._runes.id_to_name_and_id(self._data["data"]["runes"][0]["id"])
        runes = []
        for rune in self._data["data"]["runes"][0]["primary_rune_ids"]:
            runes.append(self._runes.id_to_name_and_id(rune))

        for rune in self._data["data"]["runes"][0]["secondary_rune_ids"]:
            runes.append(self._runes.id_to_name_and_id(rune))
        stat_mods = []
        for stat_mod in self._data["data"]["runes"][0]["stat_mod_ids"]:
            stat_mods.append(self._runes.id_to_name_and_id(stat_mod))
        return {
            "role": self.info["role"],
            "paths": rune_paths,
            "keystone": keystone,
            "runes": runes,
            "statMods": stat_mods
        }


def get_script_dir():
    return os.path.dirname(os.path.realpath(__file__))


def save_champion_to_json(data, file_path: str):
    with open(file_path, "w", encoding="utf-8") as f:
        output = JSON.dumps(data, indent=2)
        f.write(output)


class APIQuery:
    def api_url_builder(self, buildId, champ, role):
        return f"https://www.op.gg/_next/data/{buildId}/champions/{champ}/{role}/runes.json?region=global&tier=platinum_plus"

    def get_champion(self, build_id: str, champion: str, file_name=None):
        champion_versions = []
        if os.path.isfile(os.path.join(get_script_dir(), "data/", f"{champion}.json")):
            return
        for role in roles:
            print(f"Querying data for {champion} (role: {role}) ...")
            try:
                res = urllib3.request("GET", self.api_url_builder(build_id, champion, role))
                payload = res.json()
                champion_versions.append(APIChampion(payload))
            except Exception as e:
                print(f"Failed to process '{champion}' in role '{role}': {e}")
                return
        data = {
            "name": champion_versions[0].info["name"],
            "current_patch": champion_versions[0].current_patch["version"],
            "roles": [champ.optimal_runes for champ in champion_versions]
        }

        if file_name is None:
            file_name = champion_versions[0].info['name']
        save_champion_to_json(data,
                              os.path.join(get_script_dir(),
                              "data/", f"{file_name}.json"))

    def get_all(self, build_id: str, champions, should_async=False):
        if should_async:
            pool = Pool(2)
            for champion in champions:
                pool.apply_async(self.get_champion, (build_id, champion))
            pool.close()
            pool.join()
        else:
            for champion_name, champion_api_representation in champions.items():
                self.get_champion(build_id, champion_api_representation, file_name=champion_name)


class Scraper:
    def site_url_builder(self, champ, role):
        return f"https://www.op.gg/champions/{champ}/{role}/runes?region=global&tier=platinum_plus"

    def read_html(self, url: str):
        return urllib.request.urlopen(url).read()

    def get_latest_buildId(self):
        # We just want to know the buildId, so any champ suffices
        html = self.read_html(self.site_url_builder("blitzcrank", "support"))
        soup = BeautifulSoup(html, "html.parser")
        json_data = str(soup.find(id="__NEXT_DATA__"))
        return re.search(r'"buildId":"(\S*?)","', str(json_data)).group(1)
