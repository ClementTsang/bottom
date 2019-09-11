use heim_common::{prelude::StreamExt, units::thermodynamic_temperature};

#[derive(Clone)]
pub struct TempData {
	pub component_name : Box<str>,
	pub temperature : f32,
}

pub async fn get_temperature_data() -> Result<Vec<TempData>, heim::Error> {
	let mut temperature_vec : Vec<TempData> = Vec::new();

	let mut sensor_data = heim::sensors::temperatures();
	while let Some(sensor) = sensor_data.next().await {
		if let Ok(sensor) = sensor {
			temperature_vec.push(TempData {
				component_name : Box::from(sensor.unit()),
				temperature : sensor.current().get::<thermodynamic_temperature::degree_celsius>(), // TODO: Allow for toggling this!
			});
		}
	}

	// By default, sort temperature, then by alphabetically!  Allow for configuring this...

	// Note we sort in reverse here; we want greater temps to be higher priority.
	temperature_vec.sort_by(|a, b| {
		if a.temperature > b.temperature {
			std::cmp::Ordering::Less
		}
		else if a.temperature < b.temperature {
			std::cmp::Ordering::Greater
		}
		else {
			std::cmp::Ordering::Equal
		}
	});

	temperature_vec.sort_by(|a, b| {
		if a.component_name > b.component_name {
			std::cmp::Ordering::Greater
		}
		else if a.component_name < b.component_name {
			std::cmp::Ordering::Less
		}
		else {
			std::cmp::Ordering::Equal
		}
	});

	Ok(temperature_vec)
}
