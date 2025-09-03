//! DLX library to solve exact cover problems and generate nodes.

mod nodes;
mod blocks;

pub use nodes::*;
pub use blocks::*;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn blocks() {
        let mut game = Game2D::from_strings(3, 2, vec![
            "##",
            "##\n.#",
            "#",
        ]);

        let mat = game.get_matrix();
        let sols = Node::solve_all(&mat);
        for (i, sol) in sols.iter().enumerate() {
            print!("\n===solution {}===", i);
            for j in sol {
                print!("\n    ");
                mat[*j][game.blocks.len()..].iter().for_each(|x| print!("{}", if *x { '#' } else { '.' }));
            }
        }
    }
}
