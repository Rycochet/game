//! How a container's weight is written under its grid: `12.34kg / 45.00kg`, or
//! `3.20kg` with no limit. Kept here so every container writes it the same way.

/// What is in a container and the most it may hold, both in grams.
pub fn weight_words(grams: u32, most: Option<u32>) -> String {
    match most {
        Some(most) => format!("{} / {}", kilograms(grams), kilograms(most)),
        None => kilograms(grams),
    }
}

fn kilograms(grams: u32) -> String {
    format!("{:.2}kg", grams as f32 / 1000.0)
}

#[cfg(test)]
mod tests;
