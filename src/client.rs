#[derive(Default,Debug)]
pub struct Client {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool
}