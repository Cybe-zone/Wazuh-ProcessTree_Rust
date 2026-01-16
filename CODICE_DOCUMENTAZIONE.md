# Wazuh Process Tree Viewer - Documentazione Tecnica Dettagliata

## Indice

1. [Panoramica del Progetto](#panoramica-del-progetto)
2. [Architettura del Sistema](#architettura-del-sistema)
3. [Struttura delle Directory](#struttura-delle-directory)
4. [Frontend - Componenti Yew](#frontend---componenti-yew)
5. [Modelli Dati](#modelli-dati)
6. [Graph Renderer](#graph-renderer)
7. [Funzionalità Principali](#funzionalità-principali)
8. [Flusso dei Dati](#flusso-dei-dati)
9. [Backend API](#backend-api)
10. [Guida alla Modifica](#guida-alla-modifica)

---

## Panoramica del Progetto

**Wazuh Process Tree Viewer** è un'applicazione web per la visualizzazione e analisi di alberi dei processi in tempo reale, progettata per integrare i dati di sicurezza di Wazuh. L'applicazione è completamente scritta in Rust, utilizzando:

- **Frontend**: Yew framework (WebAssembly)
- **Backend**: Actix-web
- **Rendering**: Canvas 2D API tramite web-sys
- **Styling**: CSS personalizzato con supporto temi chiaro/scuro

### Caratteristiche Principali

- Visualizzazione interattiva di alberi di processi gerarchici
- Timeline temporale con navigazione e selezione range
- Classificazione processi (Normal, Suspicious, Critical)
- Filtri interattivi per classificazione
- Zoom e pan canvas con gesture mouse e wheel
- Pannelli laterali per dettagli nodi e legenda
- Sistema di autenticazione JWT

---

## Architettura del Sistema

```
┌─────────────────────────────────────────────────┐
│              Browser (WASM)                     │
│  ┌──────────────────────────────────────────┐  │
│  │     Yew Components (Frontend)            │  │
│  │  - App (State Management)                │  │
│  │  - GraphView (Canvas Rendering)          │  │
│  │  - Timeline (Time Navigation)            │  │
│  │  - Controls (UI Controls)                │  │
│  │  - Sidebar (Details & Legend)            │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
                      ↕ HTTP/JSON
┌─────────────────────────────────────────────────┐
│         Actix-web Server (Backend)              │
│  ┌──────────────────────────────────────────┐  │
│  │  - Authentication (JWT)                  │  │
│  │  - Process Tree API                      │  │
│  │  - Static File Serving                   │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
                      ↕
┌─────────────────────────────────────────────────┐
│            Wazuh Data Source                    │
└─────────────────────────────────────────────────┘
```

---

## Struttura delle Directory

```
Wazuh-ProcessTree_Rust/
├── frontend/               # Applicazione Yew (WASM)
│   ├── src/
│   │   ├── components/     # Componenti UI Yew
│   │   │   ├── app.rs              # Componente principale e state management
│   │   │   ├── graph_view.rs       # Canvas rendering e interazioni
│   │   │   ├── timeline.rs         # Timeline interattiva
│   │   │   ├── controls.rs         # Barra controlli superiore
│   │   │   ├── sidebar.rs          # Pannelli laterali
│   │   │   ├── details_panel.rs    # Dettagli nodo selezionato
│   │   │   └── mod.rs
│   │   ├── graph.rs        # Logica rendering grafo (GraphRenderer)
│   │   ├── models.rs       # Strutture dati condivise
│   │   ├── api.rs          # Client API HTTP
│   │   ├── lib.rs          # Entry point libreria
│   │   └── main.rs         # Entry point WASM
│   ├── styles.css          # Stili globali
│   ├── index.html          # HTML template
│   └── Trunk.toml          # Configurazione build Trunk
│
├── backend/                # Server Actix-web
│   ├── src/
│   │   ├── main.rs         # Entry point server
│   │   ├── auth.rs         # Autenticazione JWT
│   │   └── models.rs       # Modelli backend
│   └── Cargo.toml
│
└── Cargo.toml             # Workspace root
```

---

## Frontend - Componenti Yew

### 1. App Component (`frontend/src/components/app.rs`)

Il componente `App` è il cuore dell'applicazione, gestisce tutto lo stato globale e coordina i vari componenti.

#### Struttura dello State

```rust
pub struct App {
    // Dati albero processi
    tree_data: Option<ProcessTree>,          // Dati filtrati correntemente visualizzati
    full_tree_data: Option<ProcessTree>,     // Dati completi originali (backup)

    // Gestione UI
    theme: Theme,                             // Light o Dark
    left_sidebar_open: bool,                  // Visibilità sidebar sinistra (legenda)
    right_sidebar_open: bool,                 // Visibilità sidebar destra (dettagli)
    selected_node_id: Option<String>,         // ID nodo selezionato

    // Timeline e filtri
    timeline_visible: bool,                   // Visibilità timeline
    time_filter: Option<(String, String)>,    // Range temporale attivo (start, end)

    // Classificazioni nascoste
    hidden_classifications: std::collections::HashSet<String>,

    // Triggers visualizzazione
    fit_view_trigger: bool,                   // Toggle per centrare vista
    zoom_to_node_id: Option<String>,          // ID nodo per zoom specifico

    // Stato loading/errori
    loading: bool,
    alert: Option<(String, AlertType)>,

    // API client
    api_client: Rc<ApiClient>,
}
```

#### Messaggi (Msg Enum)

```rust
pub enum Msg {
    // Gestione dati
    LoadData,
    DataLoaded(ProcessTree),
    DataLoadError(String),

    // Interazioni UI
    ToggleTheme,
    ToggleSidebar(SidebarSide),
    SelectNode(Option<String>),
    ToggleClassification(String),  // Mostra/nascondi classificazione

    // Timeline
    ToggleTimeline,
    HandleTimelineAction(TimelineAction),

    // Vista
    FitView,

    // Auth
    Logout,
}
```

#### Funzioni Chiave

##### `apply_classification_filter()`
Filtra i nodi dell'albero in base alle classificazioni nascoste:

```rust
fn apply_classification_filter(&mut self) {
    if let Some(full_tree) = &self.full_tree_data {
        let mut filtered_tree = full_tree.clone();

        // Rimuove nodi con classificazioni nascoste
        filtered_tree.nodes.retain(|node| {
            !self.hidden_classifications.contains(&node.classification)
        });

        // Rimuove edges orfani (che puntano a nodi rimossi)
        let visible_node_ids: HashSet<String> =
            filtered_tree.nodes.iter().map(|n| n.id.clone()).collect();

        filtered_tree.edges.retain(|edge| {
            visible_node_ids.contains(&edge.from) &&
            visible_node_ids.contains(&edge.to)
        });

        self.tree_data = Some(filtered_tree);
        self.fit_view_trigger = !self.fit_view_trigger;  // Ricentra vista
    }
}
```

##### `update()` - Message Handler

Gestisce tutti i messaggi dell'applicazione:

```rust
fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
        Msg::ToggleClassification(classification) => {
            // Toggle classificazione nel set
            if self.hidden_classifications.contains(&classification) {
                self.hidden_classifications.remove(&classification);
            } else {
                self.hidden_classifications.insert(classification);
            }
            // Applica filtro
            self.apply_classification_filter();
            true  // Re-render
        }

        Msg::HandleTimelineAction(action) => {
            match action {
                TimelineAction::ZoomToTimeRange(start, end) => {
                    // Filtra nodi per range temporale
                    self.time_filter = Some((start.clone(), end.clone()));
                    // Logica filtro temporale...
                }

                TimelineAction::ZoomToNode(node_id) => {
                    // Imposta zoom su nodo specifico
                    self.selected_node_id = Some(node_id.clone());
                    self.zoom_to_node_id = Some(node_id);
                    // Rimuove filtro temporale
                    self.time_filter = None;
                    self.tree_data = self.full_tree_data.clone();
                }
            }
            true
        }
        // ... altri handlers
    }
}
```

##### `render_legend()` - Renderizza Legenda

Crea la legenda con pulsanti toggle per ogni classificazione:

```rust
fn render_legend(&self, ctx: &Context<Self>) -> Html {
    // Conta processi per classificazione da full_tree_data
    let mut classification_counts: HashMap<String, (usize, String)> = HashMap::new();

    if let Some(tree) = &self.full_tree_data {
        for node in &tree.nodes {
            let entry = classification_counts
                .entry(node.classification.clone())
                .or_insert((0, node.color.background.clone()));
            entry.0 += 1;
        }
    }

    // Ordina per nome
    let mut items: Vec<_> = classification_counts.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));

    // Renderizza items
    items.iter().map(|(name, (count, color))| {
        let is_hidden = self.hidden_classifications.contains(name);

        html! {
            <div class={classes!(
                "legend-item",
                if is_hidden { Some("legend-item-hidden") } else { None }
            )}>
                <div class="legend-color"
                     style={format!("background: {}", color)} />
                <div class="legend-info">
                    <div class="legend-name">{name}</div>
                    <div class="legend-count">
                        {format!("{} processes", count)}
                    </div>
                </div>
                <button
                    class="legend-toggle-btn"
                    onclick={ctx.link().callback(move |_|
                        Msg::ToggleClassification(name.clone())
                    )}
                >
                    {if is_hidden { "👁" } else { "👁‍🗨" }}
                </button>
            </div>
        }
    }).collect::<Html>()
}
```

---

### 2. GraphView Component (`frontend/src/components/graph_view.rs`)

Gestisce il canvas HTML5 e le interazioni utente (pan, zoom, selezione).

#### Struttura State

```rust
pub struct GraphView {
    canvas_ref: NodeRef,                          // Riferimento al canvas DOM
    renderer: Rc<RefCell<Option<GraphRenderer>>>, // Renderer del grafo
    animation_handle: Option<Interval>,           // Loop rendering 60fps
    last_data_hash: Option<usize>,                // Hash per detectare cambiamenti dati
    last_fit_view_trigger: bool,                  // Tracking trigger fit view
    last_zoom_node_id: Option<String>,            // Tracking zoom nodo
}
```

#### Props

```rust
pub struct Props {
    pub tree_data: Option<ProcessTree>,    // Dati da visualizzare
    pub is_light_mode: bool,               // Tema
    pub on_node_select: Callback<Option<String>>, // Callback selezione
    pub fit_view_trigger: bool,            // Trigger centra vista
    pub zoom_to_node_id: Option<String>,   // Trigger zoom nodo
}
```

#### Lifecycle Hooks

##### `rendered()` - Post-Render Logic

Eseguito dopo ogni render, gestisce aggiornamenti dati e triggers:

```rust
fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
    if first_render {
        ctx.link().send_message(Msg::CanvasReady);
    }

    // Detecta cambiamenti dati
    if let Some(tree) = &ctx.props().tree_data {
        let current_hash = tree.nodes.len() * 1000 + tree.edges.len();

        if self.last_data_hash != Some(current_hash) {
            log::info!("Tree data changed - updating renderer");
            if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                renderer.set_data(tree.nodes.clone(), tree.edges.clone());
            }
            self.last_data_hash = Some(current_hash);
        }
    }

    // Handle fit view trigger (toggle boolean)
    if ctx.props().fit_view_trigger != self.last_fit_view_trigger {
        if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
            renderer.fit_view();
        }
        self.last_fit_view_trigger = ctx.props().fit_view_trigger;
    }

    // Handle zoom to node trigger
    if let Some(node_id) = &ctx.props().zoom_to_node_id {
        if self.last_zoom_node_id.as_ref() != Some(node_id) {
            log::info!("Zoom to node trigger: {}", node_id);
            if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                renderer.zoom_to_node(node_id);
            }
            self.last_zoom_node_id = Some(node_id.clone());
        }
    } else if self.last_zoom_node_id.is_some() {
        self.last_zoom_node_id = None;
    }

    // Ridimensiona canvas se necessario
    if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
        if let Some(parent) = canvas.parent_element() {
            let width = parent.client_width() as u32;
            let height = parent.client_height() as u32;

            if canvas.width() != width || canvas.height() != height {
                canvas.set_width(width);
                canvas.set_height(height);
            }
        }
    }
}
```

#### Event Handlers

```rust
fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
        Msg::MouseDown(event) => {
            if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                let old_selected = renderer.get_selected_node()
                    .map(|n| n.id.clone());

                renderer.handle_mouse_down(&event);

                let new_selected = renderer.get_selected_node()
                    .map(|n| n.id.clone());

                // Notifica cambio selezione
                if old_selected != new_selected {
                    ctx.props().on_node_select.emit(new_selected);
                }
            }
            true
        }

        Msg::MouseMove(event) => {
            if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                renderer.handle_mouse_move(&event);
            }
            true
        }

        Msg::Wheel(event) => {
            if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                renderer.handle_wheel(&event);
            }
            true
        }
        // ...
    }
}
```

#### Animation Loop

Rendering a 60fps tramite `Interval`:

```rust
fn start_animation(&mut self, ctx: &Context<Self>) {
    let link = ctx.link().clone();

    // Interval a 16ms (~60fps)
    let interval = Interval::new(16, move || {
        link.send_message(Msg::Animate);
    });

    self.animation_handle = Some(interval);
}

// Nel message handler:
Msg::Animate => {
    if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
        renderer.render(ctx.props().is_light_mode);
    }
    false  // Non re-renderizzare componente Yew
}
```

---

### 3. Timeline Component (`frontend/src/components/timeline.rs`)

Timeline interattiva con dots cliccabili e selection range draggable.

#### Struttura State

```rust
pub struct Timeline {
    // Selection range (percentuali 0-100)
    selection_start: f64,
    selection_end: f64,
    dragging_handle: Option<DraggingHandle>,
}

enum DraggingHandle {
    Start,  // Handle inizio range
    End,    // Handle fine range
}
```

#### Props

```rust
pub struct Props {
    pub nodes: Vec<ProcessNode>,                    // Tutti i nodi
    pub is_visible: bool,                           // Visibilità
    pub on_action: Callback<TimelineAction>,        // Callback azioni
    pub hidden_classifications: HashSet<String>,    // Classificazioni nascoste
}

pub enum TimelineAction {
    ZoomToTimeRange(String, String),  // RFC3339 timestamps
    ZoomToNode(String),               // Node ID
}
```

#### Rendering Timeline Dots

```rust
fn view(&self, ctx: &Context<Self>) -> Html {
    // Filtra e parsa timestamp
    let mut timed_nodes: Vec<(ProcessNode, DateTime<Utc>)> = Vec::new();

    for node in ctx.props().nodes.iter() {
        // Salta classificazioni nascoste
        if ctx.props().hidden_classifications.contains(&node.classification) {
            continue;
        }

        // Normalizza timestamp (gestisce +0000 formato)
        let normalized_timestamp = normalize_timestamp(&node.timestamp);

        if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized_timestamp) {
            timed_nodes.push((node.clone(), dt.with_timezone(&Utc)));
        }
    }

    // Ordina cronologicamente
    timed_nodes.sort_by(|a, b| a.1.cmp(&b.1));

    let first_time = timed_nodes.first().unwrap().1;
    let last_time = timed_nodes.last().unwrap().1;
    let time_range = last_time.signed_duration_since(first_time);

    // Renderizza dots
    timed_nodes.iter().map(|(node, dt)| {
        // Calcola posizione percentuale
        let elapsed = dt.signed_duration_since(first_time);
        let position = if time_range.num_milliseconds() > 0 {
            (elapsed.num_milliseconds() as f64 /
             time_range.num_milliseconds() as f64) * 100.0
        } else {
            0.0
        };

        html! {
            <div
                class="timeline-dot"
                style={format!(
                    "left: {}%; background-color: {}",
                    position,
                    node.color.background
                )}
                title={format!(
                    "{}: {} at {}",
                    node.process_name,
                    node.process_id,
                    dt.format("%H:%M:%S%.3f")
                )}
                onclick={
                    let node_id = node.id.clone();
                    let on_action = ctx.props().on_action.clone();
                    Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        on_action.emit(TimelineAction::ZoomToNode(node_id.clone()));
                    })
                }
            />
        }
    }).collect::<Html>()
}
```

#### Selection Range Draggable

##### Rendering Handles

```rust
// Selection range overlay (sfondo verde semi-trasparente)
<div
    class="timeline-selection-range"
    style={format!(
        "left: {}%; width: {}%",
        self.selection_start,
        self.selection_end - self.selection_start
    )}
/>

// Handle inizio
<div
    class="timeline-handle timeline-handle-start"
    style={format!("left: {}%", self.selection_start)}
    onmousedown={
        let link = ctx.link().clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            link.send_message(TimelineMsg::StartDragHandle(
                DraggingHandle::Start,
                0.0
            ));
        })
    }
>
    <div class="timeline-handle-grip"></div>
</div>

// Handle fine (simile)
```

##### Drag Logic

```rust
fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
        TimelineMsg::StartDragHandle(handle, _) => {
            self.dragging_handle = Some(handle);
            true
        }

        TimelineMsg::DragHandle(position) => {
            if let Some(handle) = &self.dragging_handle {
                match handle {
                    DraggingHandle::Start => {
                        // Minimo 5% dalla fine, massimo 100%
                        self.selection_start = position
                            .max(0.0)
                            .min(self.selection_end - 5.0);
                    }
                    DraggingHandle::End => {
                        // Minimo 5% dall'inizio, massimo 100%
                        self.selection_end = position
                            .min(100.0)
                            .max(self.selection_start + 5.0);
                    }
                }
            }
            true
        }

        TimelineMsg::EndDrag => {
            if self.dragging_handle.is_some() {
                self.dragging_handle = None;

                // Calcola range temporale da percentuali
                // Ricostruisce timed_nodes, calcola first_time e time_range
                let start_offset_ms = (time_range.num_milliseconds() as f64
                    * self.selection_start / 100.0) as i64;
                let end_offset_ms = (time_range.num_milliseconds() as f64
                    * self.selection_end / 100.0) as i64;

                let start_time = first_time + Duration::milliseconds(start_offset_ms);
                let end_time = first_time + Duration::milliseconds(end_offset_ms);

                // Emetti azione
                ctx.props().on_action.emit(
                    TimelineAction::ZoomToTimeRange(
                        start_time.to_rfc3339(),
                        end_time.to_rfc3339()
                    )
                );
            }
            true
        }
    }
}
```

##### Mouse Event Handlers su Track

```rust
<div
    class="timeline-track"
    onmousemove={
        let link = ctx.link().clone();
        Callback::from(move |e: MouseEvent| {
            if e.buttons() == 1 {  // Tasto sinistro premuto
                let target = e.current_target().unwrap();
                let element: web_sys::HtmlElement = target.dyn_into().unwrap();
                let rect = element.get_bounding_client_rect();
                let mouse_x = e.client_x() as f64 - rect.left();
                let track_width = rect.width();
                let position_percent = (mouse_x / track_width * 100.0)
                    .max(0.0)
                    .min(100.0);
                link.send_message(TimelineMsg::DragHandle(position_percent));
            }
        })
    }
    onmouseup={
        let link = ctx.link().clone();
        Callback::from(move |_e: MouseEvent| {
            link.send_message(TimelineMsg::EndDrag);
        })
    }
>
```

---

### 4. Controls Component (`frontend/src/components/controls.rs`)

Barra superiore con statistiche e controlli globali.

#### Props e Rendering

```rust
pub struct Props {
    pub theme: Theme,
    pub on_toggle_theme: Callback<()>,
    pub on_fit_view: Callback<()>,
    pub on_logout: Callback<()>,
    pub classification_stats: HashMap<String, usize>,  // Conteggi per classificazione
    pub total_processes: usize,
    pub orphan_count: usize,
}

#[function_component(Controls)]
pub fn controls(props: &Props) -> Html {
    html! {
        <div class="controls-bar">
            <div class="controls-left">
                <h1 class="app-title">{"Wazuh Process Tree Viewer"}</h1>
            </div>

            <div class="controls-stats">
                // Statistiche totali
                <div class="stat-badge">
                    <span class="stat-label">{"Total Processes:"}</span>
                    <span class="stat-value">{props.total_processes}</span>
                </div>

                // Statistiche per classificazione (dinamiche)
                {
                    props.classification_stats.iter().map(|(classification, count)| {
                        let class_name = match classification.as_str() {
                            "Critical" => "stat-critical",
                            "Suspicious" => "stat-suspicious",
                            _ => "stat-normal"
                        };
                        html! {
                            <div class="stat-badge">
                                <span class="stat-label">
                                    {format!("{}:", classification)}
                                </span>
                                <span class={classes!("stat-value", class_name)}>
                                    {count}
                                </span>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>

            <div class="controls-right">
                // Pulsanti azioni...
            </div>
        </div>
    }
}
```

---

## Graph Renderer

### GraphRenderer (`frontend/src/graph.rs`)

Classe core per il rendering del grafo su canvas.

#### Struttura Dati

```rust
pub struct GraphRenderer {
    // Canvas e contesto
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,

    // Dati grafo
    nodes: Vec<NodePosition>,      // Nodi con posizioni calcolate
    edges: Vec<ProcessEdge>,       // Archi

    // Trasformazione viewport
    offset_x: f64,                 // Offset X (pan)
    offset_y: f64,                 // Offset Y (pan)
    scale: f64,                    // Zoom (1.0 = 100%)

    // Interazione
    dragging: bool,                // Pan attivo
    last_mouse_x: f64,
    last_mouse_y: f64,
    selected_node: Option<usize>,  // Indice nodo selezionato

    // Layout
    node_spacing_x: f64,           // Spaziatura orizzontale nodi
    node_spacing_y: f64,           // Spaziatura verticale nodi

    // Performance
    last_render_time: f64,
}

struct NodePosition {
    node: ProcessNode,
    x: f64,      // Coordinate mondo (non schermo)
    y: f64,
    width: f64,
    height: f64,
}
```

#### Layout Algorithm

##### `calculate_layout()` - Hierarchical Layout

```rust
fn calculate_layout(&mut self) {
    if self.nodes.is_empty() {
        return;
    }

    // 1. Identifica root nodes (senza parent)
    let mut roots = Vec::new();
    let mut has_parent: HashSet<String> = HashSet::new();

    for edge in &self.edges {
        has_parent.insert(edge.to.clone());
    }

    for node_pos in &self.nodes {
        if !has_parent.contains(&node_pos.node.id) {
            roots.push(node_pos.node.id.clone());
        }
    }

    // 2. Build adjacency list (parent -> children)
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &self.edges {
        children_map
            .entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
    }

    // 3. Traversal BFS per assegnare livelli
    let mut levels: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    for root in &roots {
        queue.push_back((root.clone(), 0));
        levels.insert(root.clone(), 0);
    }

    while let Some((node_id, level)) = queue.pop_front() {
        if let Some(children) = children_map.get(&node_id) {
            for child_id in children {
                if !levels.contains_key(child_id) {
                    levels.insert(child_id.clone(), level + 1);
                    queue.push_back((child_id.clone(), level + 1));
                }
            }
        }
    }

    // 4. Raggruppa nodi per livello
    let max_level = levels.values().max().copied().unwrap_or(0);
    let mut nodes_by_level: Vec<Vec<String>> = vec![Vec::new(); max_level + 1];

    for (node_id, level) in &levels {
        nodes_by_level[*level].push(node_id.clone());
    }

    // 5. Assegna posizioni
    for (level, node_ids) in nodes_by_level.iter().enumerate() {
        let count = node_ids.len();

        for (index, node_id) in node_ids.iter().enumerate() {
            let node_pos = self.nodes.iter_mut()
                .find(|n| &n.node.id == node_id)
                .unwrap();

            // X: distribuiti uniformemente per livello
            node_pos.x = index as f64 * self.node_spacing_x;

            // Y: basato sul livello gerarchico
            node_pos.y = level as f64 * self.node_spacing_y;
        }
    }
}
```

#### Rendering

##### `render()` - Main Render Loop

```rust
pub fn render(&mut self, is_light_mode: bool) {
    let width = self.canvas.width() as f64;
    let height = self.canvas.height() as f64;

    // 1. Clear canvas
    self.context.clear_rect(0.0, 0.0, width, height);

    // 2. Imposta colori tema
    let (bg_color, text_color, edge_color) = if is_light_mode {
        ("#ffffff", "#1a1a1a", "#cccccc")
    } else {
        ("#1a1a1a", "#e0e0e0", "#444444")
    };

    self.context.set_fill_style(&JsValue::from_str(bg_color));
    self.context.fill_rect(0.0, 0.0, width, height);

    // 3. Salva stato e applica trasformazione
    self.context.save();
    self.context.translate(self.offset_x, self.offset_y).unwrap();
    self.context.scale(self.scale, self.scale).unwrap();

    // 4. Disegna edges
    self.context.set_stroke_style(&JsValue::from_str(edge_color));
    self.context.set_line_width(2.0 / self.scale);

    for edge in &self.edges {
        if let (Some(from_node), Some(to_node)) = (
            self.nodes.iter().find(|n| n.node.id == edge.from),
            self.nodes.iter().find(|n| n.node.id == edge.to)
        ) {
            self.context.begin_path();

            // Da centro-bottom del parent
            let start_x = from_node.x + from_node.width / 2.0;
            let start_y = from_node.y + from_node.height;

            // A centro-top del child
            let end_x = to_node.x + to_node.width / 2.0;
            let end_y = to_node.y;

            // Bezier curve per edge curvo
            let control_y = (start_y + end_y) / 2.0;

            self.context.move_to(start_x, start_y);
            self.context.bezier_curve_to(
                start_x, control_y,
                end_x, control_y,
                end_x, end_y
            );
            self.context.stroke();
        }
    }

    // 5. Disegna nodi
    for (index, node_pos) in self.nodes.iter().enumerate() {
        let is_selected = self.selected_node == Some(index);

        // Background nodo
        self.context.set_fill_style(
            &JsValue::from_str(&node_pos.node.color.background)
        );

        // Arrotonda angoli
        self.draw_rounded_rect(
            node_pos.x, node_pos.y,
            node_pos.width, node_pos.height,
            8.0
        );
        self.context.fill();

        // Border per selezione
        if is_selected {
            self.context.set_stroke_style(&JsValue::from_str("#76b947"));
            self.context.set_line_width(3.0 / self.scale);
            self.context.stroke();
        }

        // Testo
        self.context.set_fill_style(&JsValue::from_str(text_color));
        self.context.set_font(&format!("{}px Arial", 14.0 / self.scale));

        let text_x = node_pos.x + 10.0;
        let text_y = node_pos.y + 20.0;

        self.context.fill_text(
            &node_pos.node.process_name,
            text_x, text_y
        ).unwrap();

        // PID
        self.context.set_font(&format!("{}px Arial", 11.0 / self.scale));
        self.context.fill_text(
            &format!("PID: {}", node_pos.node.process_id),
            text_x, text_y + 15.0
        ).unwrap();
    }

    // 6. Ripristina trasformazione
    self.context.restore();
}
```

#### Interazioni

##### `handle_mouse_down()` - Click Detection

```rust
pub fn handle_mouse_down(&mut self, event: &MouseEvent) {
    let rect = self.canvas.get_bounding_client_rect();
    let canvas_x = event.client_x() as f64 - rect.left();
    let canvas_y = event.client_y() as f64 - rect.top();

    // Trasforma coordinate schermo -> mondo
    let world_x = (canvas_x - self.offset_x) / self.scale;
    let world_y = (canvas_y - self.offset_y) / self.scale;

    // Hit test su nodi (reverse order per Z-index)
    let mut found = None;
    for (index, node_pos) in self.nodes.iter().enumerate().rev() {
        if world_x >= node_pos.x &&
           world_x <= node_pos.x + node_pos.width &&
           world_y >= node_pos.y &&
           world_y <= node_pos.y + node_pos.height {
            found = Some(index);
            break;
        }
    }

    if found.is_some() {
        self.selected_node = found;
    } else {
        // Inizia pan
        self.dragging = true;
        self.last_mouse_x = canvas_x;
        self.last_mouse_y = canvas_y;
    }
}
```

##### `handle_mouse_move()` - Pan

```rust
pub fn handle_mouse_move(&mut self, event: &MouseEvent) {
    if !self.dragging {
        return;
    }

    let rect = self.canvas.get_bounding_client_rect();
    let canvas_x = event.client_x() as f64 - rect.left();
    let canvas_y = event.client_y() as f64 - rect.top();

    // Delta movimento
    let dx = canvas_x - self.last_mouse_x;
    let dy = canvas_y - self.last_mouse_y;

    // Applica offset
    self.offset_x += dx;
    self.offset_y += dy;

    // Aggiorna posizione
    self.last_mouse_x = canvas_x;
    self.last_mouse_y = canvas_y;
}
```

##### `handle_wheel()` - Zoom

```rust
pub fn handle_wheel(&mut self, event: &WheelEvent) {
    event.prevent_default();

    let delta = event.delta_y();
    let zoom_factor = if delta < 0.0 { 1.1 } else { 0.9 };

    // Mouse position
    let rect = self.canvas.get_bounding_client_rect();
    let mouse_x = event.client_x() as f64 - rect.left();
    let mouse_y = event.client_y() as f64 - rect.top();

    // Zoom verso mouse cursor
    // world_pos prima dello zoom
    let world_x_before = (mouse_x - self.offset_x) / self.scale;
    let world_y_before = (mouse_y - self.offset_y) / self.scale;

    // Applica zoom
    self.scale *= zoom_factor;
    self.scale = self.scale.clamp(0.1, 3.0);

    // Aggiusta offset per mantenere world_pos sotto mouse
    self.offset_x = mouse_x - world_x_before * self.scale;
    self.offset_y = mouse_y - world_y_before * self.scale;
}
```

##### `zoom_to_node()` - Zoom su Nodo Specifico

```rust
pub fn zoom_to_node(&mut self, node_id: &str) {
    // Trova nodo
    let node_pos = self.nodes.iter().find(|n| n.node.id == node_id);

    if let Some(node) = node_pos {
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;

        // Calcola centro nodo (coordinate mondo)
        let node_center_x = node.x + node.width / 2.0;
        let node_center_y = node.y + node.height / 2.0;

        // Imposta zoom 1.5x
        self.scale = 1.5;

        // Centra nodo su canvas
        self.offset_x = canvas_width / 2.0 - node_center_x * self.scale;
        self.offset_y = canvas_height / 2.0 - node_center_y * self.scale;

        log::info!("Zoomed to node {} at ({}, {})",
            node_id, node_center_x, node_center_y);
    }
}
```

##### `fit_view()` - Auto-fit Tutti i Nodi

```rust
pub fn fit_view(&mut self) {
    if self.nodes.is_empty() {
        return;
    }

    // Calcola bounding box di tutti i nodi
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in &self.nodes {
        min_x = min_x.min(node.x);
        min_y = min_y.min(node.y);
        max_x = max_x.max(node.x + node.width);
        max_y = max_y.max(node.y + node.height);
    }

    let content_width = max_x - min_x;
    let content_height = max_y - min_y;

    let canvas_width = self.canvas.width() as f64;
    let canvas_height = self.canvas.height() as f64;

    // Calcola scale per fit (con padding 10%)
    let scale_x = (canvas_width * 0.9) / content_width;
    let scale_y = (canvas_height * 0.9) / content_height;
    self.scale = scale_x.min(scale_y).clamp(0.1, 3.0);

    // Centra contenuto
    let content_center_x = (min_x + max_x) / 2.0;
    let content_center_y = (min_y + max_y) / 2.0;

    self.offset_x = canvas_width / 2.0 - content_center_x * self.scale;
    self.offset_y = canvas_height / 2.0 - content_center_y * self.scale;
}
```

---

## Modelli Dati

### Core Models (`frontend/src/models.rs`)

```rust
// Nodo del grafo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessNode {
    pub id: String,                    // UUID
    pub process_id: i64,               // PID
    pub process_name: String,
    pub classification: String,        // "Normal", "Suspicious", "Critical"
    pub color: NodeColor,
    pub parent_id: Option<i64>,        // PID parent (None se root)
    pub timestamp: String,             // RFC3339 format
}

// Colori nodo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeColor {
    pub background: String,   // Hex color (es. "#76b947")
    pub border: String,
    pub text: String,
}

// Arco del grafo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessEdge {
    pub from: String,  // ID nodo parent
    pub to: String,    // ID nodo child
}

// Processo orfano (parent non trovato)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrphanProcess {
    pub process_id: i64,
    pub process_name: String,
    pub parent_id: Option<i64>,
    pub timestamp: String,
}

// Albero completo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessTree {
    pub nodes: Vec<ProcessNode>,
    pub edges: Vec<ProcessEdge>,
    pub orphans: Vec<OrphanProcess>,
    pub total_processes: usize,
}

// Statistiche
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
    pub total_processes: usize,
    pub suspicious_count: usize,
    pub orphan_count: usize,
}
```

---

## Funzionalità Principali

### 1. Filtro per Classificazione

**Flusso**:
1. User clicca toggle button su legend item (app.rs:render_legend)
2. Emit `Msg::ToggleClassification(classification)`
3. Update handler aggiunge/rimuove classificazione da `hidden_classifications` HashSet
4. Chiama `apply_classification_filter()`
5. Filtra nodi e edges da `full_tree_data`
6. Aggiorna `tree_data` con versione filtrata
7. Toggle `fit_view_trigger` per ricentrare
8. Timeline riceve `hidden_classifications` come prop e filtra dots automaticamente

### 2. Timeline Selection Range

**Flusso**:
1. User clicca e trascina handle Start o End
2. `onmousedown` su handle → `TimelineMsg::StartDragHandle`
3. `onmousemove` su track → `TimelineMsg::DragHandle(position)`
4. Update handler aggiorna `selection_start` o `selection_end`
5. Re-render aggiorna posizione visuale handle e overlay
6. `onmouseup` → `TimelineMsg::EndDrag`
7. Calcola timestamps da percentuali
8. Emit `TimelineAction::ZoomToTimeRange(start, end)`
9. App riceve azione e filtra nodi per timestamp
10. Aggiorna `tree_data` e `time_filter`

### 3. Zoom su Nodo da Timeline

**Flusso**:
1. User clicca dot su timeline
2. `onclick` emit `TimelineAction::ZoomToNode(node_id)`
3. App riceve azione
4. Imposta `selected_node_id = node_id`
5. Imposta `zoom_to_node_id = node_id`
6. Rimuove `time_filter` (mostra tutti i nodi)
7. Ripristina `tree_data` da `full_tree_data`
8. GraphView riceve `zoom_to_node_id` come prop
9. `rendered()` detecta cambio e chiama `renderer.zoom_to_node(node_id)`
10. GraphRenderer trova nodo, imposta scale 1.5x, centra nodo

### 4. Pan e Zoom Canvas

**Pan**:
- MouseDown su canvas (non su nodo) → `dragging = true`
- MouseMove con dragging → calcola delta, aggiorna `offset_x/y`
- MouseUp → `dragging = false`

**Zoom**:
- WheelEvent → calcola zoom_factor (1.1 o 0.9)
- Calcola posizione mondo sotto mouse
- Aggiorna `scale`
- Ricalcola `offset_x/y` per mantenere punto sotto mouse fisso

---

## Flusso dei Dati

### Caricamento Iniziale

```
1. App.create()
   └─> Msg::LoadData

2. ApiClient.fetch_process_tree()
   └─> GET /api/process-tree

3. Backend response (ProcessTree JSON)
   └─> Msg::DataLoaded(tree)

4. Update handler:
   full_tree_data = tree.clone()
   tree_data = tree
   loading = false

5. Render:
   - Controls riceve statistics
   - GraphView riceve tree_data
   - Timeline riceve nodes
   - Sidebars ricevono dati
```

### Update Cycle (Classificazione Toggle)

```
1. User click toggle button
   └─> Msg::ToggleClassification("Suspicious")

2. Update handler:
   - Toggle in hidden_classifications HashSet
   - apply_classification_filter()
     - Clone full_tree_data
     - Filter nodes (rimuove "Suspicious")
     - Filter edges (rimuove orfani)
     - Set tree_data = filtered
     - Toggle fit_view_trigger

3. Re-render:
   - GraphView riceve nuovo tree_data
     └─> rendered() detecta hash change
         └─> renderer.set_data()
             └─> calculate_layout()

   - GraphView riceve fit_view_trigger diverso
     └─> rendered() chiama renderer.fit_view()

   - Timeline riceve hidden_classifications
     └─> view() filtra dots

4. Animation loop continua rendering
```

### Timeline Range Selection

```
1. User drag handle
   └─> onmousedown → TimelineMsg::StartDragHandle
   └─> onmousemove → TimelineMsg::DragHandle(75.0)
   └─> onmouseup → TimelineMsg::EndDrag

2. Timeline.update():
   - DragHandle: aggiorna selection_start/end
   - EndDrag:
     - Calcola timestamps da percentuali
     - Emit TimelineAction::ZoomToTimeRange(start, end)

3. App riceve TimelineAction:
   - Msg::HandleTimelineAction(ZoomToTimeRange)
   - Parse timestamps
   - Filter tree_data per range temporale
   - Set time_filter = (start, end)

4. GraphView aggiorna con tree_data filtrato
   - rendered() detecta cambio
   - renderer.set_data()
   - calculate_layout()
```

---

## Backend API

### Endpoints (`backend/src/main.rs`)

```rust
// Login
POST /api/login
Body: { "username": "...", "password": "..." }
Response: { "token": "JWT_TOKEN" }

// Get Process Tree
GET /api/process-tree
Headers: Authorization: Bearer JWT_TOKEN
Response: ProcessTree JSON

// Logout (optional)
POST /api/logout
Headers: Authorization: Bearer JWT_TOKEN
```

### Authentication Middleware

```rust
// JWT validation
async fn validate_token(req: &HttpRequest) -> Result<Claims, Error> {
    let auth_header = req.headers()
        .get("Authorization")
        .ok_or(ErrorUnauthorized("Missing token"))?;

    let token = auth_header.to_str()
        .map_err(|_| ErrorUnauthorized("Invalid token"))?
        .strip_prefix("Bearer ")
        .ok_or(ErrorUnauthorized("Invalid format"))?;

    decode_jwt(token)
}
```

---

## Guida alla Modifica

### Aggiungere una Nuova Classificazione

1. **Backend**: Modifica logica classificazione per includere nuova categoria
2. **Frontend - models.rs**: Nessuna modifica necessaria (classificazioni dinamiche)
3. **Frontend - app.rs**: `render_legend()` genera automaticamente items
4. **Frontend - graph.rs**: Assicurati che colori siano assegnati correttamente
5. **CSS**: Aggiungi classe `stat-{nome}` se vuoi colore specifico in statistiche

### Modificare Layout Grafo

**File**: `frontend/src/graph.rs`

**Funzione**: `calculate_layout()`

Esempio - Cambio spaziatura:
```rust
impl GraphRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // ...
        node_spacing_x: 300.0,  // Era 250.0
        node_spacing_y: 150.0,  // Era 120.0
        // ...
    }
}
```

Esempio - Layout radiale invece di gerarchico:
```rust
fn calculate_layout(&mut self) {
    let center_x = 400.0;
    let center_y = 300.0;
    let radius = 200.0;

    for (index, node_pos) in self.nodes.iter_mut().enumerate() {
        let angle = (index as f64 / self.nodes.len() as f64) * 2.0 * PI;
        node_pos.x = center_x + radius * angle.cos();
        node_pos.y = center_y + radius * angle.sin();
    }
}
```

### Aggiungere Nuova Azione Timeline

1. **Timeline.rs** - Aggiungi variant a `TimelineAction`:
```rust
pub enum TimelineAction {
    ZoomToTimeRange(String, String),
    ZoomToNode(String),
    HighlightPath(String, String),  // NUOVO
}
```

2. **Timeline.rs** - Aggiungi UI trigger:
```rust
// Nel rendering
html! {
    <button onclick={
        let on_action = ctx.props().on_action.clone();
        Callback::from(move |_| {
            on_action.emit(TimelineAction::HighlightPath(
                start_node_id.clone(),
                end_node_id.clone()
            ));
        })
    }>
        {"Highlight Path"}
    </button>
}
```

3. **App.rs** - Gestisci azione:
```rust
Msg::HandleTimelineAction(action) => {
    match action {
        TimelineAction::HighlightPath(start_id, end_id) => {
            // Implementa logica
            self.highlighted_path = Some((start_id, end_id));
            // ...
        }
        // ...
    }
    true
}
```

4. **Graph.rs** - Modifica rendering:
```rust
pub fn render(&mut self, is_light_mode: bool, highlighted_path: &Option<(String, String)>) {
    // ...
    for edge in &self.edges {
        // Controlla se edge è nel path evidenziato
        let is_highlighted = highlighted_path.as_ref()
            .map(|(start, end)| {
                // Logica per determinare se edge è nel percorso start->end
                edge_in_path(&edge, start, end)
            })
            .unwrap_or(false);

        if is_highlighted {
            self.context.set_stroke_style(&JsValue::from_str("#FFD700")); // Oro
            self.context.set_line_width(4.0 / self.scale);
        } else {
            self.context.set_stroke_style(&JsValue::from_str(edge_color));
            self.context.set_line_width(2.0 / self.scale);
        }

        // Disegna edge...
    }
}
```

### Ottimizzare Performance Rendering

**Problema**: Troppi nodi rallentano rendering

**Soluzione 1 - Culling**: Renderizza solo nodi visibili

```rust
pub fn render(&mut self, is_light_mode: bool) {
    // ...

    // Calcola viewport bounds in coordinate mondo
    let view_min_x = -self.offset_x / self.scale;
    let view_max_x = (self.canvas.width() as f64 - self.offset_x) / self.scale;
    let view_min_y = -self.offset_y / self.scale;
    let view_max_y = (self.canvas.height() as f64 - self.offset_y) / self.scale;

    for node_pos in &self.nodes {
        // Skip nodi fuori viewport
        if node_pos.x + node_pos.width < view_min_x ||
           node_pos.x > view_max_x ||
           node_pos.y + node_pos.height < view_min_y ||
           node_pos.y > view_max_y {
            continue;
        }

        // Renderizza nodo...
    }
}
```

**Soluzione 2 - Level of Detail**: Semplifica rendering quando zoomed out

```rust
if self.scale < 0.5 {
    // Rendering semplificato: solo rettangolo colorato
    self.context.fill_rect(
        node_pos.x, node_pos.y,
        node_pos.width, node_pos.height
    );
} else {
    // Rendering completo con testo
    self.draw_rounded_rect(...);
    self.context.fill_text(...);
}
```

### Modificare Tema Colori

**File**: `frontend/styles.css`

**Variabili CSS**:
```css
:root {
    /* Background */
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --bg-tertiary: #3a3a3a;

    /* Testo */
    --text-primary: #e0e0e0;
    --text-secondary: #b0b0b0;
    --text-tertiary: #808080;

    /* Accenti */
    --accent-primary: #76b947;    /* Verde Wazuh */
    --accent-secondary: #5a9335;

    /* Bordi */
    --border-color: #404040;

    /* Stati */
    --color-critical: #ff4444;
    --color-suspicious: #ff9800;
    --color-normal: #4caf50;
}

.light-mode {
    --bg-primary: #ffffff;
    --bg-secondary: #f5f5f5;
    --bg-tertiary: #e0e0e0;

    --text-primary: #1a1a1a;
    --text-secondary: #666666;
    --text-tertiary: #999999;

    --border-color: #d0d0d0;
}
```

### Debug Tips

**Frontend Logging**:
```rust
// In qualsiasi componente
log::info!("Debug info: {:?}", some_value);
log::warn!("Warning message");
log::error!("Error: {}", error_msg);
```

**Browser Console**: Apri DevTools → Console per vedere i log

**Performance Profiling**:
```rust
let start = web_sys::window()
    .unwrap()
    .performance()
    .unwrap()
    .now();

// Codice da profilare

let end = web_sys::window()
    .unwrap()
    .performance()
    .unwrap()
    .now();

log::info!("Execution time: {}ms", end - start);
```

**Canvas Debugging**:
```rust
// Aggiungi in graph.rs render()
// Disegna bounding boxes
self.context.set_stroke_style(&JsValue::from_str("#ff00ff"));
self.context.stroke_rect(
    node_pos.x, node_pos.y,
    node_pos.width, node_pos.height
);

// Disegna coordinate
self.context.fill_text(
    &format!("({:.0}, {:.0})", node_pos.x, node_pos.y),
    node_pos.x, node_pos.y - 5.0
).unwrap();
```

---

## Build e Deploy

### Development

```bash
# Backend
cd backend
cargo run

# Frontend (in altra terminale)
cd frontend
trunk serve
```

### Production Build

```bash
# Frontend
cd frontend
trunk build --release

# Backend
cd backend
cargo build --release

# Deploy dist/ e backend binary
```

### Environment Variables

**Backend** (`.env`):
```
JWT_SECRET=your_secret_key_here
DATABASE_URL=postgresql://...
WAZUH_API_URL=https://...
```

---

## Conclusione

Questa documentazione copre l'architettura completa dell'applicazione Wazuh Process Tree Viewer. Per ogni modifica:

1. Identifica il componente/modulo coinvolto
2. Leggi la sezione relativa in questa documentazione
3. Modifica il codice seguendo i pattern esistenti
4. Testa localmente con `trunk serve`
5. Verifica console browser per errori
6. Commit e push

Per domande o chiarimenti, consulta i commenti nel codice o crea una issue su GitHub.
