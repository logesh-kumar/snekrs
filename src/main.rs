mod game;

fn main() -> std::io::Result<()> {
    let mut game = game::Game::new();
    game.run()?;
    Ok(())
}