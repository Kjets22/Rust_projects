
fn apply_spring_layout(&mut self) {
    thread::sleep(Duration::from_millis(100));
    self.graph_complete = false;
    // let start_time = Instant::now(); // Start timing
    // let mut change = egui::Vec2::ZERO;
    // let mut last_change = egui::Vec2::ZERO;
    let width = 800.0;
    let height = 600.0;
    let num_nodes = self.graph.len() as f32;

    // let iterations = 1000 * self.graph.len();
    let k = (width * height / (self.graph.len() as f32)).sqrt() * 0.2;
    let c = 0.005;
    // let mut number = 0;

    // while number != iterations && !converged {
    //     number += 1;
    for i in 0..self.graph.len() {
        self.forces[i] = egui::Vec2::ZERO;
    }

    for row in 0..self.graph.len() {
        for col in row + 1..self.graph.len() {
            let delta = self.positions[row] - self.positions[col];
            let distance = delta.length().max(0.01);
            let repulsive_force = (k * k) / distance * (10.0 / (num_nodes * num_nodes));
            self.forces[row] += delta.normalized() * repulsive_force * 2.0;
        }
    }

    for node in &self.graph {
        for &link in &node.links {
            let delta = self.positions[node.id] - self.positions[link];
            let distance = delta.length().max(0.01);
            let attractive_force = (distance * distance) / k * 0.5;
            self.forces[node.id] -= delta.normalized() * attractive_force;
            self.forces[link] += delta.normalized() * attractive_force;
        }
    }

    for i in 0..self.graph.len() {
        for j in (i + 1)..self.graph.len() {
            let delta = self.positions[i] - self.positions[j];
            let distance = delta.length().max(0.01);
            let min_distance = 100.0;

            if distance < min_distance {
                let overlap_force = (min_distance - distance) * 0.5;
                self.forces[i] += delta.normalized() * overlap_force * 2.0;
                self.forces[j] -= delta.normalized() * overlap_force * 2.0;
            }
        }
    }

    for i in 0..self.graph.len() {
        for node in &self.graph {
            for &link in &node.links {
                let a = self.positions[node.id];
                let b = self.positions[link];
                if node.id != i && link != i {
                    let c = self.positions[i];

                    let ab = b - a;
                    let ac = c - a;
                    let t = ac.dot(ab) / ab.dot(ab);

                    let t = t.clamp(0.0, 1.0);
                    let closest_point = a + ab * t;
                    let delta = c - closest_point;
                    let distance = delta.length().max(0.01);
                    let min_distance = 30.0;

                    if distance < min_distance {
                        let link_force = (min_distance - distance) * 0.5;
                        self.forces[i] += delta.normalized() * link_force;
                    }
                }
            }
        }

        self.positions[i] += self.forces[i] * c;
        self.change += self.forces[i];
    }

    let totalchange1 = (self.last_change[0].abs() - self.change[0].abs()).abs();
    let totalchange2 = (self.last_change[1].abs() - self.change[1].abs()).abs();
    let sumtch = totalchange1 + totalchange2;

    if (sumtch > -0.0024 && sumtch < 0.0024) && sumtch != 0.0 {
        println!("{:?}", sumtch);
        self.graph_complete = true;
        println!("convered");
        println!("{:?}", self.change);
    }
    self.last_change = self.change;
    self.change = egui::Vec2::ZERO;

    // Redraw the UI to reflect the changes
    //ctx.request_repaint();
    //thread::sleep(Duration::from_millis(100));
    //}
    println!("running");
    println!("{:?}", sumtch);
    // let end_time = Instant::now(); // End timing
    // self.layout_time = (end_time - start_time).as_secs_f64(); // Store the elapsed time
    // self.graph_complete = true;
}
