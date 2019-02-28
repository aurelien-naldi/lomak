use std::error::Error;

lazy_static! {
    static ref NO_ALIASES: Vec<String> = vec!();
    static ref NO_ARGUMENTS: Vec<Argument> = vec!();
}


pub trait Service: Sync {
    fn name(&self) -> &str;
    fn descr(&self) -> &str;
    fn aliases(&self) -> &Vec<String> {
        &NO_ALIASES
    }
    fn arguments(&self) -> &Vec<Argument> {
        &NO_ARGUMENTS
    }
}

pub trait Callable<T, E: Error> {
    fn call(&self) -> Result<T, E>;
}

pub trait Runnable {
    fn run(self);
}

pub struct Argument {
    pub name: String,
    pub descr: String,
    pub short: String,
    pub long: String,
    pub value: bool,
    pub multiple: bool,
}

impl Argument {
    pub fn new(name: &str) -> Self {
        Argument {
            name: String::from(name),
            descr: String::new(),
            short: String::new(),
            long: String::new(),
            value: false,
            multiple: false,
        }
    }

    pub fn descr(mut self, descr: &str) -> Self {
        self.descr = String::from(descr);
        self
    }
    pub fn short(mut self, short: &str) -> Self {
        self.short = String::from(short);
        self
    }
    pub fn long(mut self, long: &str) -> Self {
        self.long = String::from(long);
        self
    }

    pub fn value(mut self, value: bool) -> Self {
        self.value = value;
        self
    }
    pub fn multiple(mut self, b: bool) -> Self {
        self.multiple = b;
        self
    }
}
