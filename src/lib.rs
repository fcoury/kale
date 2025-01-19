use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyboardMetadata {
    pub author: Option<String>,
    pub backcolor: Option<String>,
    pub background: Option<Background>,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub radii: Option<String>,
    pub switch_brand: Option<String>,
    pub switch_mount: Option<String>,
    pub switch_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Background {
    pub name: String,
    pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyProperties {
    // Next key only properties
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub x2: Option<f64>,
    pub y2: Option<f64>,
    pub w2: Option<f64>,
    pub h2: Option<f64>,
    pub l: Option<bool>, // stepped
    pub n: Option<bool>, // homing
    pub d: Option<bool>, // decal

    // Rotation properties
    pub r: Option<f64>,  // rotation angle
    pub rx: Option<f64>, // rotation center x
    pub ry: Option<f64>, // rotation center y

    // Properties that apply to all subsequent keys
    pub c: Option<String>, // keycap color
    pub t: Option<String>, // text color
    pub g: Option<bool>,   // ghosted
    pub a: Option<u8>,     // text alignment
    pub f: Option<u8>,     // primary font size
    pub f2: Option<u8>,    // secondary font size
    pub p: Option<String>, // profile & row
}

#[derive(Debug)]
pub struct Key {
    pub legends: Vec<String>,
    pub properties: KeyProperties,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct Keyboard {
    pub metadata: Option<KeyboardMetadata>,
    pub keys: Vec<Key>,
}

impl Keyboard {
    fn preprocess_raw_data(raw_data: &str) -> String {
        let mut processed = String::new();
        let mut lines: Vec<&str> = raw_data
            .split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        // Remove trailing commas from each line
        (0..lines.len()).for_each(|i| {
            if lines[i].ends_with(',') {
                lines[i] = &lines[i][..lines[i].len() - 1];
            }
        });

        // Join lines and wrap in array brackets
        processed.push('[');
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                processed.push(',');
            }
            processed.push_str(line);
        }
        processed.push(']');

        // Add quotes around property names in objects
        let mut result = String::new();
        let mut in_string = false;
        let mut in_object = false;
        let mut last_char: Option<char> = None;
        let mut property_name = String::new();

        processed.chars().for_each(|c| {
            match c {
                '"' => {
                    in_string = !in_string;
                    result.push(c);
                }
                '{' if !in_string => {
                    in_object = true;
                    result.push(c);
                }
                '}' if !in_string => {
                    in_object = false;
                    result.push(c);
                }
                ':' if in_object && !in_string => {
                    if !property_name.is_empty() && !property_name.starts_with('"') {
                        result.truncate(result.len() - property_name.len());
                        result.push('"');
                        result.push_str(&property_name);
                        result.push('"');
                    }
                    result.push(c);
                    property_name.clear();
                }
                ',' if in_object && !in_string => {
                    result.push(c);
                    property_name.clear();
                }
                _ => {
                    if in_object
                        && !in_string
                        && last_char.map_or(true, |ch| ch == '{' || ch == ',')
                    {
                        property_name.clear();
                    }
                    if in_object && !in_string {
                        property_name.push(c);
                    }
                    result.push(c);
                }
            }
            last_char = Some(c);
        });

        result
    }

    pub fn parse(raw_data: &str) -> Result<Self, serde_json::Error> {
        // Normalize line endings and clean up whitespace
        let raw_data = raw_data.replace("\r\n", "\n").replace('\r', "\n");
        let raw_data = raw_data.trim();

        // Preprocess the data to ensure valid JSON
        let processed_data = Self::preprocess_raw_data(raw_data);

        let mut data: Vec<Value> = serde_json::from_str(&processed_data)?;

        // Extract metadata if present
        let metadata = if !data.is_empty() && data[0].is_object() {
            let meta = data.remove(0);
            Some(serde_json::from_value(meta)?)
        } else {
            None
        };

        let mut keys = Vec::new();
        let mut current_y = 0.0;
        let mut current_properties = KeyProperties::default();

        // Process each row
        for row in data {
            let mut current_x = 0.0;
            let row_array = row.as_array().unwrap();

            for item in row_array {
                if item.is_object() {
                    // Update properties
                    let props: KeyProperties = serde_json::from_value(item.clone())?;

                    // Update persistent properties
                    if props.c.is_some() {
                        current_properties.c = props.c.clone();
                    }
                    if props.t.is_some() {
                        current_properties.t = props.t.clone();
                    }
                    if props.g.is_some() {
                        current_properties.g = props.g;
                    }
                    if props.a.is_some() {
                        current_properties.a = props.a;
                    }
                    if props.f.is_some() {
                        current_properties.f = props.f;
                    }
                    if props.f2.is_some() {
                        current_properties.f2 = props.f2;
                    }
                    if props.p.is_some() {
                        current_properties.p = props.p.clone();
                    }

                    // Store single-key properties
                    current_properties.x = props.x;
                    current_properties.y = props.y;
                    current_properties.w = props.w;
                    current_properties.h = props.h;
                    current_properties.x2 = props.x2;
                    current_properties.y2 = props.y2;
                    current_properties.w2 = props.w2;
                    current_properties.h2 = props.h2;
                    current_properties.l = props.l;
                    current_properties.n = props.n;
                    current_properties.d = props.d;
                    current_properties.r = props.r;
                    current_properties.rx = props.rx;
                    current_properties.ry = props.ry;
                } else if item.is_string() {
                    // Process key
                    let legends: Vec<String> = item
                        .as_str()
                        .unwrap()
                        .split('\n')
                        .map(String::from)
                        .collect();

                    // Apply position adjustments
                    let x = current_x + current_properties.x.unwrap_or(0.0);
                    let y = current_y + current_properties.y.unwrap_or(0.0);

                    // If the key is rotated, use the absolute x and y values
                    let (x, y) = if current_properties.r.is_some()
                        || current_properties.rx.is_some()
                        || current_properties.ry.is_some()
                    {
                        (
                            current_properties.x.unwrap_or(0.0),
                            current_properties.y.unwrap_or(0.0),
                        )
                    } else {
                        (x, y)
                    };

                    keys.push(Key {
                        legends,
                        properties: current_properties.clone(),
                        x,
                        y,
                    });

                    // Update current_x for next key
                    current_x = x + current_properties.w.unwrap_or(1.0);

                    // Reset single-key properties
                    current_properties.x = None;
                    current_properties.y = None;
                    current_properties.w = None;
                    current_properties.h = None;
                    current_properties.x2 = None;
                    current_properties.y2 = None;
                    current_properties.w2 = None;
                    current_properties.h2 = None;
                    current_properties.l = None;
                    current_properties.n = None;
                    current_properties.d = None;
                    current_properties.r = None;
                    current_properties.rx = None;
                    current_properties.ry = None;
                }
            }
            current_y += 1.0;
        }

        Ok(Keyboard { metadata, keys })
    }

    pub fn to_raw_format(&self) -> String {
        fn format_property_object(
            props: &KeyProperties,
            last_props: &KeyProperties,
        ) -> Option<String> {
            let mut parts = Vec::new();

            // For rotated keys, we want to preserve the exact order: r, rx, ry, y, x
            if props.r.is_some() || props.rx.is_some() || props.ry.is_some() {
                // This is a rotated key - use strict ordering
                if props.r != last_props.r {
                    if let Some(r) = props.r {
                        parts.push(format!("r:{}", r));
                    }
                }

                if props.rx != last_props.rx {
                    if let Some(rx) = props.rx {
                        parts.push(format!("rx:{}", rx));
                    }
                }

                if props.ry != last_props.ry {
                    if let Some(ry) = props.ry {
                        parts.push(format!("ry:{}", ry));
                    }
                }

                // Always include y and x for rotated keys
                if let Some(y) = props.y {
                    parts.push(format!("y:{}", y));
                }

                if let Some(x) = props.x {
                    parts.push(format!("x:{}", x));
                }
            } else {
                // For non-rotated keys, use the regular order
                if props.y != last_props.y {
                    if let Some(y) = props.y {
                        parts.push(format!("y:{}", y));
                    }
                }

                if props.x != last_props.x {
                    if let Some(x) = props.x {
                        parts.push(format!("x:{}", x));
                    }
                }
            }

            if parts.is_empty() {
                None
            } else {
                Some(format!("{{{}}}", parts.join(",")))
            }
        }

        let mut output = String::new();

        // Add metadata if present
        if let Some(ref metadata) = self.metadata {
            output.push_str(&serde_json::to_string(metadata).unwrap());
            output.push_str(",\n");
        }

        // Create a map to track the original row structure
        let mut row_map: HashMap<i32, Vec<&Key>> = HashMap::new();
        let mut current_row = 0;

        // First pass: group keys by their original row structure
        for key in &self.keys {
            if let Some(y) = key.properties.y {
                // Start a new row when we see a negative y offset
                if y < 0.0 {
                    current_row += 1;
                }
            }
            row_map.entry(current_row).or_default().push(key);
        }

        // Sort rows by their number
        let mut row_keys: Vec<i32> = row_map.keys().cloned().collect();
        row_keys.sort();

        let mut last_props = KeyProperties::default();
        let mut first_row = true;

        // Output each row in original order
        for row_num in row_keys {
            if let Some(row_keys) = row_map.get(&row_num) {
                if !first_row {
                    output.push_str(",\n");
                }
                first_row = false;

                output.push('[');
                let mut first_in_row = true;

                for key in row_keys {
                    if !first_in_row {
                        output.push(',');
                    }
                    first_in_row = false;

                    // Add property object if properties changed
                    if let Some(props_str) = format_property_object(&key.properties, &last_props) {
                        output.push_str(&props_str);
                        output.push(',');
                    }

                    // Add key legend
                    output.push('"');
                    output.push_str(&key.legends.join("\\n"));
                    output.push('"');

                    last_props = key.properties.clone();
                }

                output.push(']');
            }
        }

        output
    }
}
