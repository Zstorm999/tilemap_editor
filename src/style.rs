use iced::{pure::widget::button, Color};

pub enum SelectorTheme {
    Selected,
    NotSelected,
}

impl SelectorTheme {
    pub fn pick<T: PartialEq>(current: T, intended: T) -> Self {
        if current == intended {
            SelectorTheme::Selected
        } else {
            SelectorTheme::NotSelected
        }
    }
}

impl<'a> From<SelectorTheme> for Box<dyn button::StyleSheet + 'a> {
    fn from(theme: SelectorTheme) -> Self {
        match theme {
            SelectorTheme::Selected => BtSelected.into(),
            SelectorTheme::NotSelected => Default::default(),
        }
    }
}

struct BtSelected;

impl button::StyleSheet for BtSelected {
    fn active(&self) -> button::Style {
        button::Style {
            background: Color::from_rgb(0.4, 0.4, 0.4).into(),
            ..Default::default()
        }
    }
}
