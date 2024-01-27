use bevy::{
	prelude::*,
	utils::{HashMap, HashSet},
};

#[derive(Resource, Clone, Default)]
pub struct HashGrid {
	pub chunk_size: usize,
	grid: HashMap<(i8, i8), HashSet<Entity>>,
	association: HashMap<Entity, (i8, i8)>,
}

impl HashGrid {
	pub fn new(chunk_size: usize) -> Self {
		Self {
			chunk_size,
			..default()
		}
	}

	pub fn update_entity(&mut self, entity: Entity, pos: Vec2) {
		let chunk_size = self.chunk_size as f32;
		let i = (pos.y / chunk_size) as i8;
		let j = (pos.x / chunk_size) as i8;

		if let Some((previous_i, previous_j)) = self.association.get(&entity) {
			let p_i = *previous_i;
			let p_j = *previous_j;
			if i == p_i && j == p_j { return };

			if let Some(set) = self.grid.get_mut(&(p_i, p_j)) {
				set.remove(&entity);
				if set.is_empty() {
					self.grid.remove(&(p_i, p_j));
				}
			}
		}

		self.grid
			.entry((i, j))
			.or_insert(HashSet::default())
			.insert(entity);
		self.association.insert(entity, (i, j));
	}

	pub fn get_in_radius(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
		let mut result = Vec::new();

		let chunk_size = self.chunk_size as f32;
		let x_begin = pos.x - radius;
		let y_begin = pos.y - radius;
		let i_begin = (y_begin / chunk_size) as i8;
		let j_begin = (x_begin / chunk_size) as i8;

		let i_to = (radius * 2.0 / chunk_size as f32).ceil() as i8;
        let j_to = (radius * 2.0 / chunk_size as f32).ceil() as i8;

        let i_end = i_begin + i_to;
        let j_end = j_begin + j_to;

        for i in i_begin..=i_end {
            for j in j_begin..=j_end {
                if let Some(set) = self.grid.get(&(i, j)) {
                    result.extend(set.iter());
                }
            }
        }

        result
	}
}
