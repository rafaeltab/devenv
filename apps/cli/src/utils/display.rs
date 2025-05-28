use serde_json::Value;

pub trait ToDynVec<'a> {
    fn to_dyn_vec(&self) -> Vec<&dyn RafaeltabDisplayItem>;
}

impl<T> ToDynVec<'_> for Vec<T>
where
    T: RafaeltabDisplayItem,
{
    fn to_dyn_vec(&self) -> Vec<&dyn RafaeltabDisplayItem> {
        self.iter()
            .map(|x| x as &dyn RafaeltabDisplayItem)
            .collect()
    }
}

pub trait RafaeltabDisplayItem {
    fn to_json(&self) -> Value;
    fn to_pretty_string(&self) -> String;
}

pub trait RafaeltabDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>);
    fn display(&self, element: &dyn RafaeltabDisplayItem);
}

pub struct PrettyDisplay;

impl RafaeltabDisplay for PrettyDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>) {
        for element in list {
            self.display(element);
        }
    }

    fn display(&self, element: &dyn RafaeltabDisplayItem) {
        println!("{}", element.to_pretty_string())
    }
}

pub struct JsonDisplay;

impl RafaeltabDisplay for JsonDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>) {
        let json_arr: Vec<Value> = list.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert list to json"),
        };
        println!("{}", json_str);
    }

    fn display(&self, element: &dyn RafaeltabDisplayItem) {
        let json_str = match serde_json::to_string(&element.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert element to json"),
        };
        println!("{}", json_str);
    }
}

pub struct JsonPrettyDisplay;

impl RafaeltabDisplay for JsonPrettyDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>) {
        let json_arr: Vec<Value> = list.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string_pretty(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert list to json"),
        };
        println!("{}", json_str);
    }

    fn display(&self, element: &dyn RafaeltabDisplayItem) {
        let json_str = match serde_json::to_string_pretty(&element.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert element to json"),
        };
        println!("{}", json_str);
    }
}
