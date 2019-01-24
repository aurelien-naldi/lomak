use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

pub trait Service {
    fn info(&self) -> &ServiceInfo;
}

pub trait Callable<T, E: Error> {
    fn call(&self) -> Result<T, E>;
}

pub trait Runnable {
    fn run(self);
}

pub struct ServiceInfo {
    pub name: String,
    pub aliases: Vec<String>,
    pub arguments: Vec<Argument>,
    pub descr: String,
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

pub struct ServiceManager<T: Service> {
    pub data: HashMap<String, Rc<T>>,
}

impl ServiceInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            aliases: vec![],
            arguments: vec![],
            descr: String::new(),
        }
    }

    pub fn descr(mut self, descr: &str) -> Self {
        self.descr = String::from(descr);
        self
    }

    pub fn alias(mut self, alias: &str) -> Self {
        self.aliases.push(String::from(alias));
        self
    }

    pub fn argument(mut self, arg: Argument) -> Self {
        self.arguments.push(arg);
        self
    }
}

impl<T: Service> ServiceManager<T> {
    pub fn new() -> Self {
        ServiceManager {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Rc<T>> {
        match self.data.get(name) {
            None => None,
            Some(s) => Some(Rc::clone(s)),
        }
    }

    pub fn register(mut self, srv: T) -> Self {
        self.data.insert(srv.info().name.clone(), Rc::new(srv));
        self
    }

    pub fn for_each<F: Fn(&Rc<T>)>(self, f: F) {
        let t = self.data.values().for_each(f);
    }
}
