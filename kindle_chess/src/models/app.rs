use crate::models::ui::{Display, Screen};

pub struct App {
    pub display: Display,
    pub screen_stack: Vec<Box<dyn Screen>>,
}
