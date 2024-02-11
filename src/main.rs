use nannou::prelude::*;
use rand::Rng;
use std::collections::VecDeque;

struct Model {
    species_vec: Vec<AntSpecies>,
    chart_data: Vec<VecDeque<Point2>>,
    max_points: usize,
    total_population: f64,
    day_counter: usize,
}

struct AntSpecies {
    name: String,
    susceptible: f64,
    infected: f64,
    dead: f64,
    infection_rate: f64,
    hyperparasitism_rate: f64,
    infected_lifetime: Vec<i32>,
    hyperparasitized: f64,
}

impl AntSpecies {
    fn new(name: String, s: f64, i: f64, d: f64, beta: f64, hp_rate: f64) -> AntSpecies {
        AntSpecies {
            name,
            susceptible: s,
            infected: i,
            dead: d,
            infection_rate: beta,
            hyperparasitism_rate: hp_rate,
            infected_lifetime: Vec::new(),
            hyperparasitized: 0.0,
        }
    }
    fn update_infection(&mut self) {
        let mut rng = rand::thread_rng();
        // Iterate in reverse to safely remove elements without affecting unprocessed items
        for index in (0..self.infected_lifetime.len()).rev() {
            // Decrement the infected_lifetime for each infected ant
            if self.infected_lifetime[index] > 0 {
                self.infected_lifetime[index] -= 1;
                println!("Ant at index {} has {} days left", index, self.infected_lifetime[index]);
            }
    
            // Determine the fate of ants whose infection period has ended
            if self.infected_lifetime[index] == 0 {
                if rng.gen::<f64>() < self.hyperparasitism_rate {
                    // Ant becomes hyperparasitized
                    self.hyperparasitized += 1.0;
                    println!("Ant at index {} became hyperparasitized", index);
                    self.infected_lifetime.swap_remove(index); // swap_remove for efficiency
                    self.infected -= 1.0; // Update the infected count
                } else {
                    // Ant dies
                    self.dead += 1.0;
                    println!("Ant at index {} died", index);
                }
                // Remove the ant from infected pool
                self.infected_lifetime.swap_remove(index); // swap_remove for efficiency
                self.infected -= 1.0; // Update the infected count
            }
        }
    }
    
    
    
    fn infect(&mut self, new_infections: f64) {
        // If there are no susceptible individuals left, do not infect new ants
        if self.susceptible == 0.0 {
            return;
        }
        let actual_infections = new_infections.min(self.susceptible);
        let mut rng = rand::thread_rng();
        for _ in 0..actual_infections as usize {
            let lifetime = rng.gen_range(4..8); // Declare 'lifetime' variable inside the loop
            self.infected_lifetime.push(lifetime);
            println!("New Infection: Ant with {} days to live", lifetime); // Log the new infection's lifetime
        }
        self.infected += actual_infections;
        self.susceptible -= actual_infections;
    }
    
    
    
    fn sir_step(&mut self, total_population: f64, dt: f64) {
        // Calculate new infections based on the current susceptible and infected populations
        let new_infections = self.infection_rate * self.susceptible * self.infected / total_population * dt;
        // Infect new individuals
        self.infect(new_infections);
        // Update the infection status of existing infected individuals
        self.update_infection();
    }
}

fn main() {
    nannou::app(model).update(update).run();

}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1024, 768)
        .view(view)
        .build()
        .unwrap();

    let species_vec = vec![
        AntSpecies::new("Polyrhachis mesota".to_string(), 1000.0, 10.0, 0.0, 0.423, 0.1),
        AntSpecies::new("Polyrhachis wolfi".to_string(), 1000.0, 10.0, 0.0, 0.146, 0.1),
        AntSpecies::new("Polyrhachis vigilans".to_string(), 1000.0, 10.0, 0.0, 0.139, 0.1),
        AntSpecies::new("Polyrhachis debilis".to_string(), 1000.0, 10.0, 0.0, 0.098, 0.1),  
        AntSpecies::new("Polyrhachis illaudata".to_string(), 1000.0, 10.0, 0.0, 0.094, 0.1), 
    ];

    let chart_data = species_vec.iter().map(|_| VecDeque::new()).collect();
    let total_population = 5000.0;

    Model {
        species_vec,
        chart_data,
        max_points: 50,
        total_population,
        day_counter: 0
    }
}


fn update(app: &App, model: &mut Model, _update: Update) {
    let frames_per_day = 30.0; // Define how many frames represent a day in your simulation

    // Check and process the special condition at the start of each day
    if app.elapsed_frames() % frames_per_day as u64 == 0 {
     
        for species in model.species_vec.iter_mut() {
            // Special condition: No susceptible ants left but there are infected ants
            if species.susceptible < 0.0 && species.infected > 0.0 {
                let mut rng = rand::thread_rng();
                let initially_infected = species.infected_lifetime.len();
                
                // Process each infected ant
                for _ in 0..initially_infected {
                    // Always remove the first element due to potential shifts from previous removals
                    species.infected_lifetime.remove(0);

                    // Determine whether the ant becomes hyperparasitized or dies
                    if rng.gen::<f64>() < species.hyperparasitism_rate {
                        species.hyperparasitized += 1.0;
                    } else {
                     
                    }
                }
                // Ensure the infected count matches the updated infected_lifetime vector
                species.infected = species.infected_lifetime.len() as f64;

            } else {
                // Regular SIR model update for each species
                species.sir_step(model.total_population, 1.0);
            }
        }
    }
  
    // Increment the day counter at the appropriate frame
    let day_counter_frames_per_day = 450; // Assuming a specific frame rate to represent a day counter increment
    if app.elapsed_frames() % day_counter_frames_per_day as u64 == 0 {
        model.day_counter += 1;
    }
}


fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);
    let padding = 30.0;
    let win = app.window_rect();
    let species_count = model.species_vec.len() as f32;
    let spacing = win.w() / (species_count + 1.0);
    
    let middle_index = (species_count / 2.0).floor() as usize;
    let vertical_spacing = 30.0; // Adjust the spacing as needed
    // Use the smallest dimension of the window to calculate the maximum radius
    let max_dimension = win.w().min(win.h());
    let max_radius = max_dimension / (species_count + 1.0) / 2.0;
    // Determine a reasonable vertical offset based on the window height
    let vertical_offset = win.h() / (species_count * 2.0);

    for (i, species) in model.species_vec.iter().enumerate() {

        let x = map_range(
            i as f32,
            0.0,
            species_count - 1.0,
            win.left() + spacing,
            win.right() - spacing,
        );
        let y = win.y.middle();
        let y = y - (i as f32 * padding);
        let offset_multiplier = (middle_index as isize - i as isize).abs() as f32;
        let y_offset = vertical_spacing * offset_multiplier;
        // Apply the offset from the middle position
        let y = win.y.middle() - y_offset;
        let win_height = win.h(); // Get the window height
        let circle_diameter = max_radius * 2.0;
        let total_circles_height = circle_diameter * species_count;
        let space_between_circles = (win_height - total_circles_height) / (species_count + 1.0); // Space between circles
        let first_circle_y = win.top() - space_between_circles - max_radius; // Y position of the first circle
        // Draw the green circle representing the total initial population
        draw.ellipse()
            .x_y(x, y + max_radius + 10.0)
            .radius(max_radius)
            .color(GREEN);

        // Calculate the radius for the red circle, representing infected ants
        // It should be proportional to the number of infected out of the total population
        let infected_proportion = if species.susceptible + species.infected > 0.0 {
            species.infected as f32 / (species.susceptible + species.infected) as f32
        } else {
            0.0
        };
        let infected_radius = infected_proportion * max_radius;
        // Draw the red circle for infected ants
        draw.ellipse()
            .x_y(x, y + max_radius + 10.0)
            .radius(infected_radius)
            .color(RED);
       
        // Calculate the radius for the gray circle, representing dead ants
        // It should be proportional to the number of dead out of the number of infected and dead
        let dead_proportion = species.dead as f32 / (species.infected + species.dead) as f32;
        let dead_radius = dead_proportion * infected_radius; // It should grow within the red circle
        // Draw the gray circle for dead ants
        draw.ellipse()
            .x_y(x, y + max_radius + 10.0)
            .radius(dead_radius)
            .color(GRAY);
        // Display text information below each circle
        let text_info = format!(
            "{}\nSusceptible: {:.0}\nInfected: {:.0}\nDead: {:.0}\nHyperparasitized: {:.0}",
            species.name, species.susceptible, species.infected, species.dead, species.hyperparasitized
        );

        draw.text(&text_info)
            .x_y(x, y - max_radius - 10.0)  // Adjust the position as needed// Adjust the position as needed
            .color(BLACK)
            .font_size(15);
    }
    
    let text_info = format!("Day: {}", model.day_counter);
    draw.text(&text_info)
        .x_y(win.left() + 100.0, win.top() - 100.0)  // Adjust the position as needed
        .color(BLACK)
        .font_size(20);

    let label_info = format!("Susceptible: {:.0}\nInfected: {:.0}\nDead: {:.0}\nHyperparasitized: {:.0}", model.species_vec[0].susceptible, model.species_vec[0].infected, model.species_vec[0].dead, model.species_vec[0].hyperparasitized);
    let labels = [
        ("Susceptible",GREEN),
        ("Infected", RED),
        ("Dead",  GRAY),
    ];
    
    for (i, &(label, color)) in labels.iter().enumerate() {
        // Draw the color box
        draw.rect()
            .x_y(win.right() - 240.0, win.top() - 65.0 - i as f32 * 20.0)  // Adjust the position as needed
            .w_h(40.0, 15.0)
            .color(color);
    
        // Draw the label
        let label_info = format!("{} ", label);
        draw.text(&label_info)
            .x_y(win.right() - 130.0, win.top() - 60.0 - i as f32 * 20.0)  // Adjust the position as needed
            .color(BLACK)
            .font_size(20);
    }
    draw.to_frame(app, &frame).unwrap();
}
