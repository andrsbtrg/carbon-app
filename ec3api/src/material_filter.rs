extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Pragma {
    name: String,
    args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Filter {
    field: String,
    op: String,
    arg: Vec<String>,
}

impl Filter {
    pub fn new(field: &str, op: &str, arg: Vec<&str>) -> Self {
        Filter {
            field: field.to_string(),
            op: op.to_string().to_uppercase(),
            arg: arg.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialFilter {
    pragma: Vec<Pragma>,
    category: String,
    filter: Vec<Filter>,
}
impl MaterialFilter {
    pub fn get_category(&self) -> String {
        self.category.to_string()
    }
    pub fn of_category(category: &str) -> MaterialFilter {
        MaterialFilter {
            category: category.to_string(),
            pragma: vec![
                Pragma {
                    name: "eMF".to_string(),
                    args: vec!["2.0/1".to_string()],
                },
                Pragma {
                    name: "lcia".to_string(),
                    args: vec!["EF 3.0".to_string()],
                },
            ],
            filter: vec![],
        }
    }

    pub fn filter(&mut self, field: &str, op: &str, arg: Vec<&str>) {
        let filter: Filter = Filter::new(field, op, arg);
        self.filter.push(filter);
    }
}

pub fn convert(mf: &MaterialFilter) -> String {
    let mut response = "!EC3 search(\"".to_string();
    response.push_str(&mf.category);
    response.push_str("\") WHERE");

    for (i, filter) in mf.filter.iter().enumerate() {
        response.push_str("\n ");
        response.push_str(&filter.field);
        response.push_str(": ");
        response.push_str(&filter.op);
        response.push_str("(");

        // Surround each arg with double quotation marks
        let formatted_args: Vec<String> = filter
            .arg
            .iter()
            .map(|arg| format!("\"{}\"", arg))
            .collect();
        response.push_str(&formatted_args.join(", "));

        response.push_str(")");

        // Add "AND" between filters except for the last one
        if i < mf.filter.len() - 1 {
            response.push_str(" AND");
        }
    }

    response.push_str("\n!pragma ");
    for (i, pragma) in mf.pragma.iter().enumerate() {
        response.push_str(&pragma.name);
        response.push_str("(");
        response.push_str(&format!("\"{}\"", pragma.args.join(", ")));
        response.push_str(")");

        // Add a comma and space between pragmas except for the last one
        if i < mf.pragma.len() - 1 {
            response.push_str(", ");
        }
    }

    // TODO removeme
    println!("{}", &response);
    return response;
}

#[test]
fn test_material_filter() {
    let mut mf = MaterialFilter::of_category("Concrete");
    mf.filter("jurisdiction", "in", vec!["150"]);
    mf.filter("epd_types", "in", vec!["Product EPDs", "Industry EPDs"]);

    let converted = r#"!EC3 search("Concrete") WHERE
 jurisdiction: IN("150") AND
 epd_types: IN("Product EPDs", "Industry EPDs")
!pragma eMF("2.0/1"), lcia("EF 3.0")"#;

    assert_eq!(convert(&mf).as_str(), converted)
}
