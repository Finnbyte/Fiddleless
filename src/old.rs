    fn view(&self) -> Element<Message> {
        let content = match self {
            Pokedex::Loading => {
                column![text("Searching for Pokémon...").size(40),]
                    .width(Length::Shrink)
            }
            Pokedex::Loaded { pokemon } => column![
                pokemon.view(),
                button("Keep searching!").on_press(Message::Search)
            ]
            .max_width(500)
            .spacing(20)
            .align_items(Alignment::End),
            Pokedex::Errored => column![
                text("Whoops! Something went wrong...").size(40),
                button("Try again").on_press(Message::Search)
            ]
            .spacing(20)
            .align_items(Alignment::End),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
