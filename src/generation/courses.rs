use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::Serializer;
use serde::Serialize;

#[wasm_bindgen]
pub fn js_matrix_from_courses(input: JsValue) -> JsValue {
    let input: Vec<Vec<bool>> = serde_wasm_bindgen::from_value(input).unwrap();
    let sig_width = input.len();
    let width = sig_width + input[0].len();
    let mut matrix: Vec<Vec<bool>> = Vec::new();
    
    for (i, course) in input.iter().enumerate() {
        for (j, slot) in course.iter().enumerate() {
            if *slot {
                let mut entry = vec![false; width];
                entry[i] = true;
                entry[j + sig_width] = true;
                matrix.push(entry);
            }
        }
    }
    
    matrix.serialize(&Serializer::json_compatible()).unwrap()
}
