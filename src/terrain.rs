use noise::{NoiseFn, Perlin};

pub struct TerrainGenerator {
    altitude_perlin: Perlin,
    temperature_perlin: Perlin,
}

impl TerrainGenerator {
    pub fn new(seed: u64) -> Self {
        // Derive two deterministic seeds from the original seed
        let altitude_seed = seed & 0xFFFF_FFFF; // Use the lower 32 bits
        let temperature_seed = (seed >> 32) & 0xFFFF_FFFF; // Use the upper 32 bits

        // Create Perlin noise generators with derived seeds
        let altitude_perlin = Perlin::new(altitude_seed as u32);
        let temperature_perlin = Perlin::new(temperature_seed as u32);

        TerrainGenerator {
            altitude_perlin,
            temperature_perlin,
        }
    }

    pub fn generate(&self, width: usize, height: usize, num_levels: i32, base_level: f64) -> Vec<Vec<Vec<f32>>> {
        let mut terrain = vec![vec![vec![0.0; 2]; width]; height];

        for y in 0..height {
            for x in 0..width {
                // Generate altitude and temperature using separate Perlin noise generators
                let mut altitude = 0.0;
                // Use multiple levels of detail for altitude
                for level in 0..num_levels {
                    let scale = 1 << level; // Scale factor for each level
                    altitude += self.altitude_perlin.get([x as f64 / (base_level / scale as f64), y as f64 / (base_level / scale as f64)]) as f32 / (num_levels as f32);
                }
                let temperature = self.temperature_perlin.get([x as f64 / 20.0, y as f64 / 20.0]) as f32;

                terrain[y][x] = vec![altitude, temperature];
            }
        }

        terrain
    }

    // Utility function to print a 2D visualization of the terrain
    pub fn print_map(terrain: &Vec<Vec<Vec<f32>>>, index: usize) {
        let ascii_gradient = [
            ' ', '.', '`', ',', ':', ';', '-', '~', '=', '+', '*', '#', '%', '@',
        ];

        for row in terrain {
            for cell in row {
                if let Some(value) = cell.get(index) {
                    // Normalize the value to a range of 0 to 1
                    let normalized = ((*value + 1.0) / 2.0).clamp(0.0, 1.0);
                    // Map the normalized value to an ASCII character
                    let char_index = (normalized * (ascii_gradient.len() - 1) as f32).round() as usize;
                    print!("{}", ascii_gradient[char_index]);
                } else {
                    print!("?");
                }
            }
            println!(); // Newline after each row
        }
    }
}