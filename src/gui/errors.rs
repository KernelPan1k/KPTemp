use std::fmt;

pub enum KPTempErrors {
    SystemClassCreation,
    WindowCreationFail,
    ProgressBar(String),
    Button(String),
    Label(String),
    Checkbox(String),
}

impl KPTempErrors {
    fn translate(&self) -> String {
        match self {
            &KPTempErrors::SystemClassCreation => format!("Failed to create a system class for a control"),
            &KPTempErrors::WindowCreationFail => format!("Failed to create a system window for a control"),
            &KPTempErrors::ProgressBar(ref e) => format!("Progressbar: {}", e),
            &KPTempErrors::Button(ref e) => format!("Button: {}", e),
            &KPTempErrors::Label(ref e) => format!("Label: {}", e),
            &KPTempErrors::Checkbox(ref e) => format!("Checkbox: {}", e),
        }
    }
}

impl fmt::Debug for KPTempErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.translate())
    }
}