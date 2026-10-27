
mod model;

use model::{
    AttributeValue, DynamicArray, Folder, InventoryObject, LinkedList,
    ObjectValue, PersistenceEngine, TemplateAttribute,
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, State};

struct AppState {
    folders: Mutex<DynamicArray<Folder>>,
    undo: Mutex<Vec<DynamicArray<Folder>>>,
    db_path: PathBuf,
}

#[derive(Serialize)]
struct TemplateDto {
    kind: String,
    label: String,
    category: String,
    color: String,
    min_label: String,
    max_label: String,
}

#[derive(Serialize)]
struct AttributeDto {
    kind: String,
    label: String,
    value: String,
    visible: bool,
    category: String,
    color: String,
    min_label: String,
    max_label: String,
}

#[derive(Serialize)]
struct FolderDto {
    id: usize,
    name: String,
    template: Vec<TemplateDto>,
    objects: Vec<ObjectDto>,
}

#[derive(Serialize)]
struct ObjectDto {
    id: usize,
    name: String,
    description: String,
    attributes: Vec<AttributeDto>,
}

#[derive(Deserialize)]
struct AttributeInput {
    value: String,
    visible: bool,
}

fn template_to_vec(folder: &Folder) -> Vec<TemplateAttribute> {
    let mut result = Vec::new();
    let mut node = &folder.template.head;

    while let Some(current) = node {
        result.push(current.data.clone());
        node = &current.next;
    }

    result
}

fn template_dto(attribute: &TemplateAttribute) -> TemplateDto {
    match attribute {
        TemplateAttribute::Text { title } => TemplateDto {
            kind: "text".into(),
            label: title.clone(),
            category: String::new(),
            color: String::new(),
            min_label: String::new(),
            max_label: String::new(),
        },
        TemplateAttribute::SingleTag { category, color } => TemplateDto {
            kind: "tag".into(),
            label: category.clone(),
            category: category.clone(),
            color: color.clone(),
            min_label: String::new(),
            max_label: String::new(),
        },
        TemplateAttribute::TagSlider { label, min_label, max_label } => TemplateDto {
            kind: "slider".into(),
            label: label.clone(),
            category: String::new(),
            color: String::new(),
            min_label: min_label.clone(),
            max_label: max_label.clone(),
        },
        TemplateAttribute::Checkbox { label } => TemplateDto {
            kind: "checkbox".into(),
            label: label.clone(),
            category: String::new(),
            color: String::new(),
            min_label: String::new(),
            max_label: String::new(),
        },
    }
}

fn attribute_dto(
    template: &TemplateAttribute,
    attribute: &AttributeValue,
) -> AttributeDto {
    let base = template_dto(template);

    let value = match &attribute.value {
        ObjectValue::Text(v) | ObjectValue::SingleTag(v) => v.clone(),
        ObjectValue::TagSlider(v) => v.to_string(),
        ObjectValue::Checkbox(v) => v.to_string(),
    };

    AttributeDto {
        kind: base.kind,
        label: base.label,
        value,
        visible: attribute.visible,
        category: base.category,
        color: base.color,
        min_label: base.min_label,
        max_label: base.max_label,
    }
}

fn to_dto(folders: &DynamicArray<Folder>) -> Vec<FolderDto> {
    let mut result = Vec::new();

    for i in 0..folders.len() {
        if let Some(folder) = folders.get(i) {
            let templates = template_to_vec(folder);
            let template_data: Vec<TemplateDto> =
                templates.iter().map(template_dto).collect();

            let mut objects = Vec::new();

            for j in 0..folder.objects.len() {
                if let Some(object) = folder.objects.get(j) {
                    let mut attributes = Vec::new();
                    let mut value_node = &object.attributes.head;
                    let mut index = 0;

                    while let Some(node) = value_node {
                        if let Some(template) = templates.get(index) {
                            attributes.push(attribute_dto(template, &node.data));
                        }
                        index += 1;
                        value_node = &node.next;
                    }

                    let description = attributes
                        .iter()
                        .find(|attribute| attribute.kind == "text")
                        .map(|attribute| attribute.value.clone())
                        .unwrap_or_default();

                    objects.push(ObjectDto {
                        id: j,
                        name: object.title.clone(),
                        description,
                        attributes,
                    });
                }
            }

            result.push(FolderDto {
                id: i,
                name: folder.name.clone(),
                template: template_data,
                objects,
            });
        }
    }

    result
}

fn save(folders: &DynamicArray<Folder>, path: &PathBuf) -> Result<(), String> {
    PersistenceEngine::save_to_file(folders, &path.to_string_lossy())
        .map_err(|error| format!("No se pudo guardar el inventario: {error}"))
}

fn record_undo(
    history: &Mutex<Vec<DynamicArray<Folder>>>,
    folders: &DynamicArray<Folder>,
) -> Result<(), String> {
    let mut history = history
        .lock()
        .map_err(|_| "No se pudo acceder al historial".to_string())?;

    history.push(folders.clone());

    if history.len() > 30 {
        history.remove(0);
    }

    Ok(())
}

#[tauri::command]
fn get_inventory(state: State<'_, AppState>) -> Result<Vec<FolderDto>, String> {
    let folders = state.folders.lock()
        .map_err(|_| "No se pudo acceder al inventario".to_string())?;
    Ok(to_dto(&folders))
}

#[tauri::command]
fn create_folder(
    name: String,
    state: State<'_, AppState>,
) -> Result<Vec<FolderDto>, String> {
    let name = name.trim();

    if name.is_empty() {
        return Err("El nombre de la carpeta no puede estar vacío".into());
    }

    let mut folders = state.folders.lock()
        .map_err(|_| "No se pudo acceder al inventario".to_string())?;

    if (0..folders.len()).any(|i| {
        folders.get(i).is_some_and(|f| f.name.eq_ignore_ascii_case(name))
    }) {
        return Err("Ya existe una carpeta con ese nombre".into());
    }

    record_undo(&state.undo, &folders)?;

    let mut template = LinkedList::new();
    template.push_back(TemplateAttribute::Text {
        title: "Descripción".into(),
    });

    folders.append(Folder {
        name: name.to_string(),
        template,
        objects: DynamicArray::new(5),
    });

    save(&folders, &state.db_path)?;
    Ok(to_dto(&folders))
}

#[tauri::command]
fn create_object(
    folder_index: usize,
    name: String,
    attributes: Vec<AttributeInput>,
    state: State<'_, AppState>,
) -> Result<Vec<FolderDto>, String> {
    let name = name.trim();

    if name.is_empty() {
        return Err("El nombre del objeto no puede estar vacío".into());
    }

    let mut folders = state.folders.lock()
        .map_err(|_| "No se pudo acceder al inventario".to_string())?;

    let folder = folders.get(folder_index)
        .ok_or_else(|| "La carpeta seleccionada ya no existe".to_string())?;

    let templates = template_to_vec(folder);
    record_undo(&state.undo, &folders)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "No se pudo generar el ID".to_string())?
        .as_nanos();


let mut object_attributes = LinkedList::new();

for (index, template) in templates.iter().enumerate() {
    let input = &attributes[index];

    let value = match template {
        TemplateAttribute::Text { .. } => {
            ObjectValue::Text(input.value.clone())
        }
        TemplateAttribute::SingleTag { .. } => {
            ObjectValue::SingleTag(input.value.clone())
        }
        TemplateAttribute::TagSlider { .. } => {
            let number = input.value.parse::<u8>()
                .map_err(|_| "El control deslizante debe ser un número entre 0 y 100".to_string())?;

            if number > 100 {
                return Err("El control deslizante debe estar entre 0 y 100".into());
            }

            ObjectValue::TagSlider(number)
        }
        TemplateAttribute::Checkbox { .. } => {
            ObjectValue::Checkbox(input.value == "true")
        }
    };

    object_attributes.push_back(AttributeValue {
        value,
        visible: input.visible,
    });
}


    let folder = folders.get_mut(folder_index).unwrap();

    folder.objects.append(InventoryObject {
        id: timestamp.to_string(),
        title: name.to_string(),
        fallback_images: DynamicArray::new(2),
        attributes: object_attributes,
    });

    save(&folders, &state.db_path)?;
    Ok(to_dto(&folders))
}

#[tauri::command]
fn edit_object(
    folder_index: usize,
    object_index: usize,
    name: String,
    attributes: Vec<AttributeInput>,
    state: State<'_, AppState>,
) -> Result<Vec<FolderDto>, String> {
    let name = name.trim();

    if name.is_empty() {
        return Err("El nombre del objeto no puede estar vacío".into());
    }

    let mut folders = state.folders.lock()
        .map_err(|_| "No se pudo acceder al inventario".to_string())?;

    let folder = folders.get(folder_index)
        .ok_or_else(|| "La carpeta seleccionada ya no existe".to_string())?;

    let templates = template_to_vec(folder);

    if attributes.len() != templates.len() {
        return Err("La cantidad de atributos recibidos no coincide con la plantilla".into());
    }

    let folder = folders.get_mut(folder_index).unwrap();

    if folder.objects.get(object_index).is_none() {
        return Err("El objeto seleccionado ya no existe".into());
    }

    record_undo(&state.undo, &folders)?;

    let folder = folders.get_mut(folder_index).unwrap();
    let object = folder.objects.get_mut(object_index).unwrap();
    object.title = name.to_string();

    for (i, template) in templates.iter().enumerate() {
        let input = &attributes[i];

        if let Some(attribute) = object.attributes.get_mut(i) {
            attribute.visible = input.visible;

            attribute.value = match template {
                TemplateAttribute::Text { .. } => {
                    ObjectValue::Text(input.value.clone())
                }
                TemplateAttribute::SingleTag { .. } => {
                    ObjectValue::SingleTag(input.value.clone())
                }
                TemplateAttribute::TagSlider { .. } => {
                    let number = input.value.parse::<u8>()
                        .map_err(|_| "El control deslizante debe ser un número entre 0 y 100")?;
                    if number > 100 {
                        return Err("El control deslizante debe estar entre 0 y 100".into());
                    }
                    ObjectValue::TagSlider(number)
                }
                TemplateAttribute::Checkbox { .. } => {
                    ObjectValue::Checkbox(input.value == "true")
                }
            };
        }
    }

    save(&folders, &state.db_path)?;
    Ok(to_dto(&folders))
}

#[tauri::command]
fn delete_object(
    folder_index: usize,
    object_index: usize,
    state: State<'_, AppState>,
) -> Result<Vec<FolderDto>, String> {
    let mut folders = state.folders.lock()
        .map_err(|_| "No se pudo acceder al inventario".to_string())?;

    if folders.get(folder_index)
        .and_then(|folder| folder.objects.get(object_index))
        .is_none()
    {
        return Err("El objeto seleccionado ya no existe".into());
    }

    record_undo(&state.undo, &folders)?;

    let folder = folders.get_mut(folder_index).unwrap();
    folder.objects.remove(object_index);

    save(&folders, &state.db_path)?;
    Ok(to_dto(&folders))
}

#[tauri::command]
fn undo_last(state: State<'_, AppState>) -> Result<Vec<FolderDto>, String> {
    let mut folders = state.folders.lock()
        .map_err(|_| "No se pudo acceder al inventario".to_string())?;

    let previous = {
        let mut history = state.undo.lock()
            .map_err(|_| "No se pudo acceder al historial".to_string())?;

        history.pop()
            .ok_or_else(|| "No hay acciones para deshacer".to_string())?
    };

    *folders = previous;
    save(&folders, &state.db_path)?;
    Ok(to_dto(&folders))
}

fn initialize(app: &AppHandle) -> Result<AppState, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("No se encontró la carpeta de datos: {e}"))?;

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("No se pudo crear la carpeta de datos: {e}"))?;

    let db_path = data_dir.join("inventario.db");

    if !db_path.exists() {
        let initial_db =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../inventario.db");

        if initial_db.exists() {
            std::fs::copy(&initial_db, &db_path)
                .map_err(|e| format!("No se pudo copiar el inventario inicial: {e}"))?;
        }
    }

    let path_string = db_path.to_string_lossy().to_string();
    let folders = PersistenceEngine::load_from_file(&path_string)
        .map_err(|e| format!("No se pudo cargar el inventario: {e}"))?;

    Ok(AppState {
        folders: Mutex::new(folders),
        undo: Mutex::new(Vec::new()),
        db_path,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = initialize(app.handle())
                .map_err(std::io::Error::other)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_inventory,
            create_folder,
            create_object,
            edit_object,
            delete_object,
            undo_last
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación");
}
