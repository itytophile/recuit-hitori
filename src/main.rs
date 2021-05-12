use rand::{
    prelude::{SliceRandom, ThreadRng},
    Rng,
};
use std::collections::{HashMap, HashSet};

/// Je désigne tout tableau carré 2D qui contient
/// des entiers non signés de 16 bits comme des hitoris.
type Hitori<const SIZE: usize> = [[u16; SIZE]; SIZE];
/// On garde les marques dans un autre tableau de même taille.
type Marks<const SIZE: usize> = [[bool; SIZE]; SIZE];

/// Le facteur qui fera baisser la température
const RT: f32 = 0.9999;
/// Le seuil où on considère que le recuit a échoué
const MORE_FIRE_THRESHOLD: f32 = 0.01;

fn main() {
    let solution = HITORI.recuit();

    println!("\n");

    for (line, mark_line) in HITORI.iter().zip(solution.iter()) {
        for (number, &is_marked) in line.iter().zip(mark_line.iter()) {
            if is_marked {
                print!("###")
            } else {
                print!("{:3}", number)
            }
        }
        println!()
    }
}

/// Si vous voulez changer le hitori n'oubliez pas de changer
/// le 15 de Hitori<15> !
const HITORI: Hitori<15> = [
    [11, 12, 4, 11, 11, 9, 6, 10, 14, 6, 5, 6, 8, 6, 3],
    [15, 7, 2, 12, 13, 9, 1, 5, 8, 10, 9, 4, 5, 11, 9],
    [4, 15, 8, 9, 8, 1, 4, 11, 7, 6, 13, 7, 2, 4, 5],
    [10, 9, 5, 9, 15, 7, 4, 3, 7, 14, 9, 2, 9, 13, 9],
    [11, 14, 2, 6, 2, 12, 15, 2, 5, 2, 10, 2, 4, 2, 1],
    [14, 13, 8, 7, 9, 2, 7, 15, 4, 1, 7, 5, 14, 12, 4],
    [9, 2, 10, 14, 1, 6, 5, 2, 15, 12, 6, 3, 11, 6, 7],
    [7, 3, 7, 2, 10, 5, 6, 7, 4, 6, 8, 12, 10, 1, 10],
    [14, 2, 6, 4, 5, 13, 9, 1, 3, 13, 12, 2, 10, 7, 8],
    [5, 6, 1, 5, 8, 14, 6, 12, 7, 11, 7, 8, 14, 10, 4],
    [5, 10, 5, 11, 4, 7, 2, 5, 1, 7, 6, 15, 9, 7, 13],
    [13, 2, 15, 10, 12, 6, 10, 9, 2, 5, 11, 2, 3, 8, 14],
    [2, 5, 2, 1, 2, 4, 8, 13, 6, 15, 2, 10, 2, 3, 12],
    [12, 2, 7, 15, 3, 15, 14, 6, 9, 4, 2, 9, 1, 5, 9],
    [8, 1, 2, 13, 8, 10, 12, 4, 11, 2, 3, 4, 5, 2, 15],
];

/// Ce trait va nous permettre de donner des méthodes
/// aux élément de type Marks
trait MarksCoord {
    /// Retourne la liste des coordonnées des marques posées.
    /// Cela évite de s'embêter avec une matrice.
    fn get_markable_coords(&self) -> Vec<(usize, usize)>;
    /// Compte le nombre de composantes connexes tracées par les marques.
    fn get_component_count(&self) -> usize;
}

impl<const SIZE: usize> MarksCoord for Marks<SIZE> {
    fn get_markable_coords(&self) -> Vec<(usize, usize)> {
        let mut coords = Vec::with_capacity(SIZE * SIZE);
        for (line_index, line) in self.iter().enumerate() {
            for (col_index, &is_marked) in line.iter().enumerate() {
                if is_marked {
                    coords.push((line_index, col_index))
                }
            }
        }
        coords
    }

    fn get_component_count(&self) -> usize {
        let marked_count: usize = self
            .iter()
            .map(|line| line.iter().filter(|a| **a).count())
            .sum();

        let mut unvisited = HashSet::with_capacity(SIZE * SIZE - marked_count);

        for line in 0..SIZE {
            for col in 0..SIZE {
                if !self[line][col] {
                    // on convertit en i32 pour tricher plus tard
                    unvisited.insert((line as i32, col as i32));
                }
            }
        }

        // On va éviter de faire de la récursivité grâce à une pile maison
        let mut visit_stack = Vec::with_capacity(SIZE * SIZE);

        let mut component_count = 0;

        while !unvisited.is_empty() {
            component_count += 1;
            visit_stack.push(unvisited.iter().next().copied().unwrap());

            while !visit_stack.is_empty() {
                let (line, col) = visit_stack.pop().unwrap();
                unvisited.remove(&(line, col));

                // On peut regarder les coordonnées
                // sans faire attention aux limites car on ne regarde plus dans
                // un tableau et on a mis les coordonnées en entiers naturels.
                if unvisited.contains(&(line, col - 1)) {
                    visit_stack.push((line, col - 1))
                }

                if unvisited.contains(&(line, col + 1)) {
                    visit_stack.push((line, col + 1))
                }

                if unvisited.contains(&(line - 1, col)) {
                    visit_stack.push((line - 1, col))
                }

                if unvisited.contains(&(line + 1, col)) {
                    visit_stack.push((line + 1, col))
                }
            }
        }

        component_count
    }
}

/// Ce trait va nous permettre de donner des méthodes
/// aux élément de type Hitori
trait Markable<const SIZE: usize> {
    /// Retourne une matrice qui montre où sont les cases
    /// utiles à marquer. En effet cela ne sert à rien de marquer
    /// une case qui n'a pas d'occurences sur sa ligne/colonne.
    /// Cela nous permet de réduire l'espace de recherche.
    fn find_markables(&self) -> Marks<SIZE>;
    /// La fonction fitness mais plus cela retourne de points, moins c'est bien.
    /// On va compter les occurences sur les mêmes colonnes/lignes, les marques
    /// côte à côte et le nombre de composantes connexes tracées par les marques.
    /// Si le hitori est valide, cela renvoie 0.
    fn bad_points(&self, solution: &Marks<SIZE>) -> u32;
    /// La fonction recuit qui cherche une solution.
    fn recuit(&self) -> Marks<SIZE>;
    /// Convertit une liste de coordonnées de marques et une liste de booléens en
    /// une matrice de marques. La liste de booléens est généralement générée aléatoirement,
    /// cela permet de choisir parmi des possibilités de marquages (données par find_markables).
    fn generate_solution_from_markable_coords(
        markable_coords_bool: &[bool],
        markable_coords: &[(usize, usize)],
    ) -> Marks<SIZE>;
}

/// Retourne une liste aléatoire de booléens. Utilisée pour choisir aléatoirement
/// entre des possibilités de marquages.
fn get_random_bools(size: usize, rng: &mut ThreadRng) -> Vec<bool> {
    let mut bools = vec![false; size];
    for bool in bools.iter_mut() {
        if rng.gen::<f32>() < 0.5 {
            *bool = true;
        }
    }
    bools
}

/// Inverse aléatoirement les booléens d'une liste de booléens. L'argument "distance" est le nombre
/// de booléens à inverser.
fn bools_random_neighbour(bools: &[bool], rng: &mut ThreadRng, distance: usize) -> Vec<bool> {
    let mut neighbour: Vec<bool> = bools.iter().copied().collect();
    let indexes: Vec<usize> = (0..neighbour.len()).collect();
    for &index in indexes.choose_multiple(rng, distance) {
        neighbour[index] ^= true;
    }
    neighbour
}

impl<const SIZE: usize> Markable<SIZE> for Hitori<SIZE> {
    fn find_markables(&self) -> Marks<SIZE> {
        let mut markables = [[false; SIZE]; SIZE];

        // occurences horizontales
        for (line, line_mark) in self.iter().zip(markables.iter_mut()) {
            for i in 0..SIZE {
                if !line_mark[i] {
                    for j in i + 1..SIZE {
                        if line[i] == line[j] {
                            line_mark[i] = true;
                            line_mark[j] = true
                        }
                    }
                }
            }
        }

        // occurences verticales
        for col_index in 0..SIZE {
            for i in 0..SIZE {
                for j in i + 1..SIZE {
                    if self[i][col_index] == self[j][col_index] {
                        markables[i][col_index] = true;
                        markables[j][col_index] = true
                    }
                }
            }
        }

        markables
    }

    fn bad_points(&self, solution: &Marks<SIZE>) -> u32 {
        let mut points = 0;
        // on compte les occurences horizontales
        for (line, line_mark) in self.iter().zip(solution.iter()) {
            let mut occurences: HashMap<u16, u32> = HashMap::with_capacity(SIZE);
            for (number, is_marked) in line.iter().zip(line_mark.iter()) {
                if !is_marked {
                    occurences.insert(*number, occurences.get(number).copied().unwrap_or(0) + 1);
                }
            }
            for occurence in occurences.values() {
                points += occurence - 1;
            }
        }

        // on compte les occurences verticales
        for col_index in 0..SIZE {
            let mut occurences: HashMap<u16, u32> = HashMap::with_capacity(SIZE);
            for i in 0..SIZE {
                if !solution[i][col_index] {
                    occurences.insert(
                        self[i][col_index],
                        occurences.get(&self[i][col_index]).copied().unwrap_or(0) + 1,
                    );
                }
            }
            for occurence in occurences.values() {
                points += occurence - 1;
            }
        }

        // marques à côte à côte
        for line_index in 0..SIZE {
            for col_index in 0..SIZE {
                if solution[line_index][col_index] {
                    if col_index + 1 < SIZE && solution[line_index][col_index + 1] {
                        points += 1;
                    }
                    if line_index + 1 < SIZE && solution[line_index + 1][col_index] {
                        points += 1;
                    }
                }
            }
        }

        // -1 car dans un hitori idéal il y a une unique composante composante connexe
        // qu'on souhaite ignorer
        points + solution.get_component_count() as u32 - 1
    }

    fn recuit(&self) -> Marks<SIZE> {
        let mut rng = rand::thread_rng();

        // On regarde quelles sont les cases utiles à marquer
        let markables = self.find_markables();
        // On en profite pour récupérer la liste de coordonnées
        // pour se faciliter la tâche plus tard
        let markable_coords = markables.get_markable_coords();

        println!(
            "Taille de l'espace de recherche: 2^{}",
            markable_coords.len()
        );

        print!("Génération médiane...");
        let mut random_scores = Vec::with_capacity(SIZE * SIZE);

        // On génére aléatoirement des solutions pour avoir une médiane des scores.
        // Je sais pas si c'est vraiment utile, j'ai juste repris le conseil du TD.
        for _ in 0..random_scores.capacity() {
            let mut markable_coords_bool = get_random_bools(markable_coords.len(), &mut rng);
            for bool in markable_coords_bool.iter_mut() {
                if rng.gen::<f32>() < 0.5 {
                    *bool = true;
                }
            }
            random_scores.push(
                self.bad_points(&Self::generate_solution_from_markable_coords(
                    &markable_coords_bool,
                    &markable_coords,
                )),
            );
        }

        // Fonction magique de la biblio standard
        let median = *random_scores.select_nth_unstable(SIZE * SIZE / 2).1;
        // On fait la racine sinon c'est trop chaud. Je ne sais pas si c'est la meilleure
        // méthode mais ça a l'air de bien marcher pour tous les hitoris.
        let t0 = (median as f32).sqrt();
        let mut temperature = t0;

        println!("\r\rGénération médiane - OK: {}\nT0 = {:.2}", median, t0);

        let mut x = get_random_bools(markable_coords.len(), &mut rng);
        let mut points_x = self.bad_points(&Self::generate_solution_from_markable_coords(
            &x,
            &markable_coords,
        ));

        loop {
            let y = bools_random_neighbour(&x, &mut rng, 1);
            let solution_y = Self::generate_solution_from_markable_coords(&y, &markable_coords);
            let points_y = self.bad_points(&solution_y);
            if points_y == 0 {
                break solution_y;
            }
            if points_y < points_x
                || rng.gen::<f32>()
                // on sait que points_x <= points_y donc le résultat sera négatif ou nul.
                    < ((points_x as i32 - points_y as i32) as f32 / temperature).exp()
            {
                x = y;
                points_x = points_y;
                print!("\r\rscore: {}, temp: {:.2}°     ", points_x, temperature);
            }
            temperature *= RT;
            if temperature < MORE_FIRE_THRESHOLD {
                println!("\r\rMORE FIRE!!!!!!!!!        ");
                temperature = t0;
            }
        }
    }

    fn generate_solution_from_markable_coords(
        markable_coords_bool: &[bool],
        markable_coords: &[(usize, usize)],
    ) -> Marks<SIZE> {
        let mut solution = [[false; SIZE]; SIZE];
        for (&(line, col), &chosen) in markable_coords.iter().zip(markable_coords_bool.iter()) {
            if chosen {
                solution[line][col] = true;
            }
        }
        solution
    }
}
