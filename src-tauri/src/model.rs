use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

const DB_PATH: &str = "inventario.db";

// ==========================================
// MÓDULO 1: ARREGLO DINÁMICO (Dynamic Array)
// ==========================================
#[derive(Clone)]
pub struct DynamicArray<T> {
    buffer: Box<[Option<T>]>,
    length: usize,
    capacity: usize,
}

impl<T: Clone> DynamicArray<T> {
    pub fn new(initial_capacity: usize) -> Self {
        let cap = if initial_capacity == 0 { 2 } else { initial_capacity };
        let buffer = vec![None; cap].into_boxed_slice();
        Self { buffer, length: 0, capacity: cap }
    }

    fn grow(&mut self) {
        if self.length == self.capacity {
            let new_capacity = (self.capacity * 1618) / 1000 + 1;
            let mut new_buffer = vec![None; new_capacity].into_boxed_slice();
            for i in 0..self.length {
                new_buffer[i] = self.buffer[i].take();
            }
            self.buffer = new_buffer;
            self.capacity = new_capacity;
        }
    }

    pub fn append(&mut self, data: T) {
        self.grow();
        self.buffer[self.length] = Some(data);
        self.length += 1;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.length { self.buffer[index].as_ref() } else { None }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.length { self.buffer[index].as_mut() } else { None }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.length { return None; }
        let removed = self.buffer[index].take();
        for i in index..self.length - 1 {
            self.buffer[i] = self.buffer[i + 1].take();
        }
        self.length -= 1;
        removed
    }

    pub fn insert(&mut self, index: usize, data: T) {
        if index >= self.length {
            self.append(data);
            return;
        }
        self.grow();
        for i in (index..self.length).rev() {
            self.buffer[i + 1] = self.buffer[i].take();
        }
        self.buffer[index] = Some(data);
        self.length += 1;
    }

    pub fn len(&self) -> usize { self.length }
    pub fn is_empty(&self) -> bool { self.length == 0 }
}

// ==========================================
// MÓDULO 2: LISTA ENLAZADA, PILA Y COLA
// ==========================================
#[derive(Clone)]
pub struct Node<T> {
    pub data: T,
    pub next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(data: T) -> Self { Node { data, next: None } }
}

#[derive(Clone)]
pub struct LinkedList<T> {
    pub head: Option<Box<Node<T>>>,
    pub length: usize,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self { LinkedList { head: None, length: 0 } }
    pub fn is_empty(&self) -> bool { self.head.is_none() }
    pub fn len(&self) -> usize { self.length }

    pub fn push_front(&mut self, data: T) {
        let mut new_node = Box::new(Node::new(data));
        new_node.next = self.head.take();
        self.head = Some(new_node);
        self.length += 1;
    }

    pub fn push_back(&mut self, data: T) {
        let new_node = Box::new(Node::new(data));
        let mut current = &mut self.head;
        while let Some(node) = current {
            current = &mut node.next;
        }
        *current = Some(new_node);
        self.length += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.length -= 1;
            node.data
        })
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.length { return None; }
        if index == 0 { return self.pop_front(); }
        
        let mut current = self.head.as_mut().unwrap();
        for _ in 0..index - 1 {
            current = current.next.as_mut().unwrap();
        }
        let removed_node = current.next.take().unwrap();
        current.next = removed_node.next;
        self.length -= 1;
        Some(removed_node.data)
    }

    pub fn insert(&mut self, index: usize, data: T) {
        if index == 0 {
            self.push_front(data);
            return;
        }
        if index >= self.length {
            self.push_back(data);
            return;
        }
        let mut current = self.head.as_mut().unwrap();
        for _ in 0..index - 1 {
            current = current.next.as_mut().unwrap();
        }
        let mut new_node = Box::new(Node::new(data));
        new_node.next = current.next.take();
        current.next = Some(new_node);
        self.length += 1;
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let mut current = self.head.as_mut();
        let mut i = 0;
        while let Some(node) = current {
            if i == index { return Some(&mut node.data); }
            current = node.next.as_mut();
            i += 1;
        }
        None
    }
}

// Implementación de impresión solo para tipos que soportan Display
impl<T: Display> LinkedList<T> {
    pub fn print(&self) {
        println!("(Total: {})", self.length);
        let mut current = &self.head;
        let mut i = 0;
        while let Some(node) = current {
            println!("  [{}] {}", i, node.data);
            current = &node.next;
            i += 1;
        }
    }
}

pub struct Queue<T> { pub list: LinkedList<T> }
impl<T> Queue<T> {
    pub fn new() -> Self { Queue { list: LinkedList::new() } }
    pub fn add(&mut self, data: T) { self.list.push_back(data); }
    pub fn pop(&mut self) -> Option<T> { self.list.pop_front() }
    pub fn is_empty(&self) -> bool { self.list.is_empty() }
}

pub struct Stack<T> { pub list: LinkedList<T> }
impl<T> Stack<T> {
    pub fn new() -> Self { Stack { list: LinkedList::new() } }
    pub fn push(&mut self, data: T) { self.list.push_front(data); }
    pub fn pop(&mut self) -> Option<T> { self.list.pop_front() }
}

// ==========================================
// MÓDULO 3: MODELOS DE DOMINIO (Inventario)
// ==========================================
#[derive(Clone, PartialEq, PartialOrd)]
pub enum TemplateAttribute {
    Text { title: String },
    SingleTag { category: String, color: String },
    TagSlider { label: String, min_label: String, max_label: String },
    Checkbox { label: String },
}

impl Display for TemplateAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TemplateAttribute::Text { title } => write!(f, "[Texto: {}]", title),
            TemplateAttribute::SingleTag { category, color } => write!(f, "[Tag {}: {}]", category, color),
            TemplateAttribute::TagSlider { label, min_label, max_label } => write!(f, "[Slider {}: {} a {}]", label, min_label, max_label),
            TemplateAttribute::Checkbox { label } => write!(f, "[Checkbox: {}]", label),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum ObjectValue {
    Text(String),
    SingleTag(String),
    TagSlider(u8),
    Checkbox(bool),
}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ObjectValue::Text(val) => write!(f, "{}", val),
            ObjectValue::SingleTag(val) => write!(f, "{}", val),
            ObjectValue::TagSlider(val) => write!(f, "{}%", val),
            ObjectValue::Checkbox(val) => write!(f, "{}", if *val { "Sí" } else { "No" }),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct AttributeValue {
    pub value: ObjectValue,
    pub visible: bool,
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let status = if self.visible { "VISIBLE" } else { "OCULTO" };
        write!(f, "{} [{}]", self.value, status)
    }
}

#[derive(Clone)]
pub struct InventoryObject {
    pub id: String,
    pub title: String,
    pub fallback_images: DynamicArray<String>, 
    pub attributes: LinkedList<AttributeValue>,
}

#[derive(Clone)]
pub struct Folder {
    pub name: String,
    pub template: LinkedList<TemplateAttribute>,
    pub objects: DynamicArray<InventoryObject>,
}

// ==========================================
// MÓDULO 4: MOTOR DE PERSISTENCIA
// ==========================================
pub struct PersistenceEngine;

impl PersistenceEngine {
    pub fn save_to_file(folders: &DynamicArray<Folder>, filepath: &str) -> std::io::Result<()> {
        let mut file = File::create(filepath)?;
        for i in 0..folders.len() {
            if let Some(folder) = folders.get(i) {
                writeln!(file, "FOLDER|{}", folder.name)?;
                let mut current_attr = &folder.template.head;
                while let Some(node) = current_attr {
                    match &node.data {
                        TemplateAttribute::Text { title } => writeln!(file, "ATTR_TEXT|{}", title)?,
                        TemplateAttribute::TagSlider { label, min_label, max_label } => writeln!(file, "ATTR_SLIDER|{}|{}|{}", label, min_label, max_label)?,
                        TemplateAttribute::Checkbox { label } => writeln!(file, "ATTR_CHECK|{}", label)?,
                        TemplateAttribute::SingleTag { category, color } => writeln!(file, "ATTR_TAG|{}|{}", category, color)?,
                    }
                    current_attr = &node.next;
                }
                for j in 0..folder.objects.len() {
                    if let Some(obj) = folder.objects.get(j) {
                        writeln!(file, "OBJECT|{}|{}", obj.id, obj.title)?;
                        let mut current_val = &obj.attributes.head;
                        while let Some(v_node) = current_val {
                            let vis = v_node.data.visible;
                            match &v_node.data.value {
                                ObjectValue::Text(t) => writeln!(file, "VAL_TEXT|{}|{}", t, vis)?,
                                ObjectValue::TagSlider(s) => writeln!(file, "VAL_SLIDER|{}|{}", s, vis)?,
                                ObjectValue::Checkbox(c) => writeln!(file, "VAL_CHECK|{}|{}", c, vis)?,
                                ObjectValue::SingleTag(tag) => writeln!(file, "VAL_TAG|{}|{}", tag, vis)?,
                            }
                            current_val = &v_node.next;
                        }
                        writeln!(file, "END_OBJECT")?;
                    }
                }
                writeln!(file, "END_FOLDER")?;
            }
        }
        Ok(())
    }

    pub fn load_from_file(filepath: &str) -> std::io::Result<DynamicArray<Folder>> {
        let file = match File::open(filepath) {
            Ok(f) => f,
            Err(_) => return Ok(DynamicArray::new(5)),
        };
        let reader = BufReader::new(file);
        let mut folders = DynamicArray::new(5);
        let mut current_folder: Option<Folder> = None;
        let mut current_object: Option<InventoryObject> = None;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split('|').collect();
            if parts.is_empty() { continue; }

            match parts[0] {
                "FOLDER" => current_folder = Some(Folder { name: parts[1].to_string(), template: LinkedList::new(), objects: DynamicArray::new(5) }),
                "ATTR_TEXT" => if let Some(f) = current_folder.as_mut() { f.template.push_back(TemplateAttribute::Text { title: parts[1].to_string() }); },
                "ATTR_SLIDER" => if let Some(f) = current_folder.as_mut() { f.template.push_back(TemplateAttribute::TagSlider { label: parts[1].to_string(), min_label: parts[2].to_string(), max_label: parts[3].to_string() }); },
                "ATTR_CHECK" => if let Some(f) = current_folder.as_mut() { f.template.push_back(TemplateAttribute::Checkbox { label: parts[1].to_string() }); },
                "ATTR_TAG" => if let Some(f) = current_folder.as_mut() { f.template.push_back(TemplateAttribute::SingleTag { category: parts[1].to_string(), color: parts[2].to_string() }); },
                "OBJECT" => current_object = Some(InventoryObject { id: parts[1].to_string(), title: parts[2].to_string(), fallback_images: DynamicArray::new(2), attributes: LinkedList::new() }),
                "VAL_TEXT" => if let Some(o) = current_object.as_mut() { o.attributes.push_back(AttributeValue { value: ObjectValue::Text(parts[1].to_string()), visible: parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(true) }); },
                "VAL_SLIDER" => if let Some(o) = current_object.as_mut() { o.attributes.push_back(AttributeValue { value: ObjectValue::TagSlider(parts[1].parse().unwrap_or(0)), visible: parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(true) }); },
                "VAL_CHECK" => if let Some(o) = current_object.as_mut() { o.attributes.push_back(AttributeValue { value: ObjectValue::Checkbox(parts[1].parse().unwrap_or(false)), visible: parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(true) }); },
                "VAL_TAG" => if let Some(o) = current_object.as_mut() { o.attributes.push_back(AttributeValue { value: ObjectValue::SingleTag(parts[1].to_string()), visible: parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(true) }); },
                "END_OBJECT" => if let Some(obj) = current_object.take() { if let Some(f) = current_folder.as_mut() { f.objects.append(obj); } },
                "END_FOLDER" => if let Some(f) = current_folder.take() { folders.append(f); },
                _ => {}
            }
        }
        Ok(folders)
    }
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

