import sys
import os
import lib
import json as JSON
import argparse


def init_parser():
    parser = argparse.ArgumentParser(
        prog="runescraper",
        description="This program queries champions' data from 'op.gg' and saves them into a JSON format locally.",
        usage="usage='%(prog)s --champion <e.g. kaisa|all> [options]'",
        epilog='Good luck!')
    parser.add_argument("--champion",
                        required=True,
                        help="Example: 'kaisa' or 'all'")
    parser.add_argument('--async', dest='should_async', action='store_true')
    parser.add_argument('--no-async', dest='should_async', action='store_false')
    parser.set_defaults(should_async=False)
    parser.add_argument("--force",
                        type=bool,
                        required=False,
                        default=False,
                        help="Determines wheather script overwrites existing champion json files")
    return parser


def create_data_dir():
    try:
        os.mkdir(os.path.join(lib.get_script_dir(), "data"))
    except FileExistsError:
        return


if __name__ == "__main__":
    args = init_parser().parse_args().__dict__
    search_term = args["champion"]
    create_data_dir()
    with open(os.path.join(lib.get_script_dir(), "champions.json"), 'r') as f:
        champions = JSON.load(f)
        scraper = lib.Scraper()
        build_id = scraper.get_latest_buildId()
        apiQuery = lib.APIQuery()
        try:
            if search_term != "all":
                champion = champions[search_term]
                apiQuery.get_champion(build_id, champion)
            elif search_term == "all":
                apiQuery.get_all(build_id, champions, should_async=args["should_async"])
        except KeyError:
            print("Champion given as input not valid.")
