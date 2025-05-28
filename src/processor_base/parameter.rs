pub trait ParameterTypeTrait : 
            std::fmt::Debug 
          + std::fmt::Display 
          + Ord 
          + Clone 
          + Copy 
          + Send 
          + Sync {}

pub trait ParameterTrait : Sized + Clone + PartialEq + Eq + PartialOrd + Ord {}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parameter<T: ParameterTypeTrait> {
    pub name: String,
    pub description: String,
    pub current_value: T,
    pub next_value: T,
    pub min_value: Option<T>,
    pub max_value: Option<T>,
    pub allowed_values: Option<Vec<T>>,
}

impl <T: ParameterTypeTrait> Parameter<T> {
    pub fn new(
        name: String,
        description: String,
        current_value: T,
        min_value: Option<T>,
        max_value: Option<T>,
        allowed_values: Option<Vec<T>>,
    ) -> Self {
        Self {
            name,
            description,
            current_value,
            next_value: current_value.clone(),
            min_value,
            max_value,
            allowed_values,
        }
    }
    pub fn set_next_value(&mut self, value: T) {
        if self.check_limits(value) == false {
            panic!("Value {} is out of limits", value);
        }
        if self.check_allowed_values(value) == false {
            panic!("Value {} is not in allowed values", value);
        }
        self.next_value = value;
    }
    pub fn update_value(&mut self) {
        self.current_value = self.next_value.clone();
    }
    pub fn check_limits(&self, value: T) -> bool {
        if let Some(min_value) = &self.min_value {
            if value < *min_value {
                return false;
            }
        }
        if let Some(max_value) = &self.max_value {
            if value > *max_value {
                return false;
            }
        }
        true
    }
    pub fn check_allowed_values(&self, value: T) -> bool {
        if let Some(allowed_values) = &self.allowed_values {
            if !allowed_values.contains(&value) {
                return false;
            }
        }
        true
    }
    pub fn get_current_value(&self) -> &T {
        &self.current_value
    }
}