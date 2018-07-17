use Game;

//generate random 2's or 4's into games
pub trait Generate {
    fn generate(&mut self);
}

impl Generate for Game {
    fn generate(&mut self) {

    }
}


