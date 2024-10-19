pub trait Model {
    pub fn update(&mut self);
    pub fn view(&self);
}
