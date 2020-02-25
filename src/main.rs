mod game;

use game::Game;

fn main() {
    let mut game = Game::new(20, 10);
    game.start();
}
