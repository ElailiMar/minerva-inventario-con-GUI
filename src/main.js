
const { invoke } = window.__TAURI__.core;

let folders = [];
let currentFolderId = 0;
let editingObjectId = null;

const $ = selector => document.querySelector(selector);

function currentFolder() {
  return folders.find(folder => folder.id === currentFolderId);
}

function escapeHtml(value) {
  return String(value).replace(/[&<>"']/g, character => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;",
    '"': "&quot;", "'": "&#039;"
  })[character]);
}

function showToast(message) {
  const toast = $("#toast");
  toast.textContent = message;
  toast.classList.add("visible");
  clearTimeout(showToast.timer);
  showToast.timer = setTimeout(() => toast.classList.remove("visible"), 2600);
}

function showError(error) {
  console.error(error);
  showToast(String(error));
}

async function refreshInventory(selectId = currentFolderId) {
  try {
    folders = await invoke("get_inventory");

    if (!folders.some(folder => folder.id === selectId)) {
      currentFolderId = folders[0]?.id ?? 0;
    } else {
      currentFolderId = selectId;
    }

    render();
  } catch (error) {
    showError(`No se pudo cargar el inventario: ${error}`);
  }
}

function renderFolders() {
  $("#folder-list").innerHTML = folders.map(folder => `
    <button class="folder-item ${folder.id === currentFolderId ? "selected" : ""}"
            data-folder-id="${folder.id}">
      <span class="folder-symbol">▰</span>
      <span>${escapeHtml(folder.name)}</span>
      <span class="folder-total">${folder.objects.length}</span>
    </button>
  `).join("");

  $("#folder-list").querySelectorAll("[data-folder-id]").forEach(button => {
    button.addEventListener("click", () => {
      currentFolderId = Number(button.dataset.folderId);
      render();
    });
  });
}

function renderObjects() {
  const folder = currentFolder();
  const query = $("#search-input").value.trim().toLowerCase();

  if (!folder) {
    $("#object-count").textContent = "0";
    $("#object-list").innerHTML = "";
    $("#empty-state").hidden = false;
    return;
  }

  const filtered = folder.objects.filter(object =>
    object.name.toLowerCase().includes(query) ||
    object.description.toLowerCase().includes(query) ||
    object.attributes.some(attribute =>
      attribute.value.toLowerCase().includes(query)
    )
  );

  $("#object-count").textContent = folder.objects.length;
  $("#object-list").innerHTML = filtered.map(object => `
    <tr>
      <td>${escapeHtml(object.name)}</td>
      <td>${escapeHtml(object.description || "Sin descripción")}</td>
      <td class="row-actions">
        <button class="button button-secondary"
                data-edit-id="${object.id}">Editar</button>
        <button class="delete-object"
                data-delete-id="${object.id}"
                title="Eliminar objeto"
                aria-label="Eliminar ${escapeHtml(object.name)}">✕</button>
      </td>
    </tr>
  `).join("");

  $("#empty-state").hidden = filtered.length > 0;

  $("#object-list").querySelectorAll("[data-edit-id]").forEach(button => {
    button.addEventListener("click", () => {
      openEditObject(Number(button.dataset.editId));
    });
  });

  $("#object-list").querySelectorAll("[data-delete-id]").forEach(button => {
    button.addEventListener("click", async () => {
      const id = Number(button.dataset.deleteId);
      const object = folder.objects.find(item => item.id === id);
      if (!object || !confirm(`¿Eliminar "${object.name}"?`)) return;

      try {
        folders = await invoke("delete_object", {
          folderIndex: folder.id,
          objectIndex: id
        });
        render();
        showToast("Objeto eliminado.");
      } catch (error) {
        showError(error);
      }
    });
  });
}

function render() {
  const folder = currentFolder();

  $("#folder-count").textContent = folders.length;
  $("#current-folder-name").textContent = folder?.name ?? "Sin carpetas";
  $("#page-title").textContent = folder
    ? `Carpeta ${folder.name}`
    : "Inventario";

  renderFolders();
  renderObjects();
}

function openDialog(id) {
  $(id).showModal();
}

document.querySelectorAll("[data-close]").forEach(button => {
  button.addEventListener("click", () => {
    $(`#${button.dataset.close}`).close();
  });
});

$("#add-folder").addEventListener("click", () => {
  $("#folder-name").value = "";
  openDialog("#folder-dialog");
  $("#folder-name").focus();
});

$("#folder-form").addEventListener("submit", async event => {
  event.preventDefault();
  const name = $("#folder-name").value.trim();
  if (!name) return;

  try {
    folders = await invoke("create_folder", { name });
    const created = folders.find(
      folder => folder.name.toLowerCase() === name.toLowerCase()
    );
    currentFolderId = created?.id ?? 0;
    $("#folder-dialog").close();
    render();
    showToast("Carpeta creada y guardada.");
  } catch (error) {
    showError(error);
  }
});

function renderAttributeFields(template, existing = []) {
  const container = $("#object-attributes");
  container.innerHTML = "";

  template.forEach((attribute, index) => {
    const current = existing[index];
    const value = current?.value ?? (
      attribute.kind === "slider" ? "0" :
      attribute.kind === "checkbox" ? "false" : ""
    );
    const visible = current?.visible ?? true;
    const field = document.createElement("div");
    field.className = "attribute-field";

    const label = document.createElement("label");
    label.textContent = attribute.label || "Atributo";
    field.appendChild(label);

    let input;

    if (attribute.kind === "checkbox") {
      input = document.createElement("input");
      input.type = "checkbox";
      input.checked = value === "true";
      input.dataset.attributeValue = "true";
      label.appendChild(input);
    } else if (attribute.kind === "slider") {
      const rangeLabel = document.createElement("div");
      rangeLabel.className = "slider-labels";
      rangeLabel.innerHTML =
        `<span>${escapeHtml(attribute.min_label)}</span>
         <span>${escapeHtml(attribute.max_label)}</span>`;
      field.appendChild(rangeLabel);

      input = document.createElement("input");
      input.type = "range";
      input.min = "0";
      input.max = "100";
      input.value = value;
      input.addEventListener("input", () => {
        field.querySelector(".range-value").textContent = input.value;
      });
      field.appendChild(input);

      const output = document.createElement("span");
      output.className = "range-value";
      output.textContent = value;
      field.appendChild(output);
    } else if (attribute.kind === "tag") {
      input = document.createElement("input");
      input.type = "text";
      input.value = value;
      input.placeholder = `Valor para ${attribute.category || attribute.label}`;
      field.appendChild(input);
    } else {
      input = document.createElement("textarea");
      input.rows = 2;
      input.value = value;
      input.placeholder = attribute.label;
      field.appendChild(input);
    }

    if (attribute.kind === "checkbox") {
      // El checkbox se añadió dentro de su etiqueta.
    } else {
      input.dataset.attributeValue = "true";
    }

    const visibilityLabel = document.createElement("label");
    visibilityLabel.className = "visibility-option";
    const visibility = document.createElement("input");
    visibility.type = "checkbox";
    visibility.checked = visible;
    visibility.dataset.attributeVisible = String(index);
    visibilityLabel.append(visibility, document.createTextNode(" Mostrar atributo"));
    field.appendChild(visibilityLabel);

    container.appendChild(field);
  });
}

function collectAttributeInputs() {
  const folder = currentFolder();

  return folder.template.map((attribute, index) => {
    const field = $("#object-attributes").children[index];
    const input = field.querySelector("[data-attribute-value]");
    const value = attribute.kind === "checkbox"
      ? String(input.checked)
      : input.value;

    return {
      value,
      visible: field.querySelector("[data-attribute-visible]").checked
    };
  });
}

function openNewObject() {
  const folder = currentFolder();
  if (!folder) {
    showToast("Primero crea una carpeta.");
    return;
  }

  editingObjectId = null;
  $("#object-name").value = "";
  $("#object-dialog-title").textContent = "Nuevo objeto";
  $("#object-submit").textContent = "Crear objeto";

  renderAttributeFields(folder.template);
  openDialog("#object-dialog");
  $("#object-name").focus();
}

function openEditObject(id) {
  const folder = currentFolder();
  const object = folder?.objects.find(item => item.id === id);
  if (!folder || !object) return;

  editingObjectId = id;
  $("#object-name").value = object.name;
  $("#object-dialog-title").textContent = "Editar objeto";
  $("#object-submit").textContent = "Guardar cambios";

  renderAttributeFields(folder.template, object.attributes);
  openDialog("#object-dialog");
  $("#object-name").focus();
}

$("#new-object").addEventListener("click", openNewObject);
$("#empty-new-object").addEventListener("click", openNewObject);

$("#object-form").addEventListener("submit", async event => {
  event.preventDefault();

  const folder = currentFolder();
  const name = $("#object-name").value.trim();
  if (!folder || !name) return;

  const attributes = collectAttributeInputs();
  const description = attributes.find((_, index) =>
    folder.template[index]?.kind === "text"
  )?.value ?? "";

  try {

if (editingObjectId === null) {
  folders = await invoke("create_object", {
    folderIndex: folder.id,
    name,
    attributes
  });

  showToast("Objeto creado y guardado.");
} else {
  folders = await invoke("edit_object", {
    folderIndex: folder.id,
    objectIndex: editingObjectId,
    name,
    attributes
  });

  showToast("Cambios guardados.");
}
    $("#object-dialog").close();
    render();
  } catch (error) {
    showError(error);
  }
});

$("#search-input").addEventListener("input", renderObjects);

$("#undo-button").addEventListener("click", async () => {
  try {
    folders = await invoke("undo_last");
    currentFolderId = folders.some(folder => folder.id === currentFolderId)
      ? currentFolderId
      : (folders[0]?.id ?? 0);
    render();
    showToast("Última acción deshecha.");
  } catch (error) {
    showError(error);
  }
});

$("#nav-inventory").addEventListener("click", () => {
  currentFolderId = folders[0]?.id ?? 0;
  render();
});

refreshInventory();
