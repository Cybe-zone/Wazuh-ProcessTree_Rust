use yew::prelude::*;
use web_sys::window;
use gloo::storage::{LocalStorage, Storage};

use crate::api::ApiClient;
use crate::models::*;
use crate::components::{graph_view::GraphView, sidebar::Sidebar, details_panel::DetailsPanel, controls::Controls};

pub enum Msg {
    ToggleTheme,
    ToggleSidebar(SidebarSide),
    SetAgentId(String),
    SetStartTime(String),
    SetEndTime(String),
    SetFilterProcess(String),
    SetFilterUser(String),
    SetFilterCmd(String),
    SetExcludeSystem(bool),
    Analyze,
    AnalysisComplete(Result<AnalysisData, String>),
    SelectNode(Option<String>),
    Logout,
    FitView,
    LoadClassifications,
    ClassificationsLoaded(Result<Vec<ProcessClassification>, String>),
}

#[derive(Clone, PartialEq)]
pub enum SidebarSide {
    Left,
    Right,
}

pub struct App {
    api: ApiClient,
    theme: Theme,
    left_sidebar_visible: bool,
    right_sidebar_visible: bool,

    // Form data
    agent_id: String,
    start_time: String,
    end_time: String,
    filter_process: String,
    filter_user: String,
    filter_cmd: String,
    exclude_system: bool,

    // Data
    tree_data: Option<ProcessTree>,
    statistics: Option<Statistics>,
    selected_node_id: Option<String>,
    classifications: Vec<ProcessClassification>,

    // UI State
    loading: bool,
    alert_message: Option<(String, AlertType)>,
    fit_view_trigger: bool,
}

#[derive(Clone, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Clone, PartialEq)]
pub enum AlertType {
    Success,
    Error,
    Info,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Load theme from localStorage
        let theme = if LocalStorage::get::<String>("theme").unwrap_or_default() == "light" {
            Theme::Light
        } else {
            Theme::Dark
        };

        // Load sidebar preferences
        let left_sidebar_visible = LocalStorage::get::<bool>("leftSidebarVisible").unwrap_or(true);
        let right_sidebar_visible = LocalStorage::get::<bool>("rightSidebarVisible").unwrap_or(true);

        // Set default times (1 hour ago to now)
        let now = js_sys::Date::new_0();
        let one_hour_ago = js_sys::Date::new(&(now.get_time() - 3600000.0).into());

        let start_time = format!(
            "{}-{:02}-{:02}T{:02}:{:02}",
            one_hour_ago.get_full_year(),
            one_hour_ago.get_month() + 1,
            one_hour_ago.get_date(),
            one_hour_ago.get_hours(),
            one_hour_ago.get_minutes()
        );

        let end_time = format!(
            "{}-{:02}-{:02}T{:02}:{:02}",
            now.get_full_year(),
            now.get_month() + 1,
            now.get_date(),
            now.get_hours(),
            now.get_minutes()
        );

        // Load classifications
        ctx.link().send_message(Msg::LoadClassifications);

        Self {
            api: ApiClient::new(),
            theme,
            left_sidebar_visible,
            right_sidebar_visible,
            agent_id: String::new(),
            start_time,
            end_time,
            filter_process: String::new(),
            filter_user: String::new(),
            filter_cmd: String::new(),
            exclude_system: false,
            tree_data: None,
            statistics: None,
            selected_node_id: None,
            classifications: Vec::new(),
            loading: false,
            alert_message: None,
            fit_view_trigger: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleTheme => {
                self.theme = match self.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
                let theme_str = match self.theme {
                    Theme::Light => "light",
                    Theme::Dark => "dark",
                };
                LocalStorage::set("theme", theme_str).ok();
                true
            }

            Msg::ToggleSidebar(side) => {
                match side {
                    SidebarSide::Left => {
                        self.left_sidebar_visible = !self.left_sidebar_visible;
                        LocalStorage::set("leftSidebarVisible", self.left_sidebar_visible).ok();
                    }
                    SidebarSide::Right => {
                        self.right_sidebar_visible = !self.right_sidebar_visible;
                        LocalStorage::set("rightSidebarVisible", self.right_sidebar_visible).ok();
                    }
                }
                true
            }

            Msg::SetAgentId(value) => {
                self.agent_id = value;
                false
            }

            Msg::SetStartTime(value) => {
                self.start_time = value;
                false
            }

            Msg::SetEndTime(value) => {
                self.end_time = value;
                false
            }

            Msg::SetFilterProcess(value) => {
                self.filter_process = value;
                false
            }

            Msg::SetFilterUser(value) => {
                self.filter_user = value;
                false
            }

            Msg::SetFilterCmd(value) => {
                self.filter_cmd = value;
                false
            }

            Msg::SetExcludeSystem(value) => {
                self.exclude_system = value;
                false
            }

            Msg::Analyze => {
                if self.agent_id.trim().is_empty() {
                    self.show_alert("Please enter an Agent ID".to_string(), AlertType::Error);
                    return true;
                }

                self.loading = true;
                let api = self.api.clone();

                let filters = if !self.filter_process.is_empty() || !self.filter_user.is_empty() || !self.filter_cmd.is_empty() {
                    Some(ProcessFilters {
                        process_name: if self.filter_process.is_empty() { None } else { Some(self.filter_process.clone()) },
                        user: if self.filter_user.is_empty() { None } else { Some(self.filter_user.clone()) },
                        command_line_contains: if self.filter_cmd.is_empty() { None } else { Some(self.filter_cmd.clone()) },
                    })
                } else {
                    None
                };

                let request = AnalysisRequest {
                    agent_id: self.agent_id.clone(),
                    start_time: format!("{}:00Z", self.start_time),
                    end_time: format!("{}:00Z", self.end_time),
                    filters,
                };

                ctx.link().send_future(async move {
                    let result = api.analyze_process(request).await;
                    Msg::AnalysisComplete(result)
                });

                true
            }

            Msg::AnalysisComplete(result) => {
                self.loading = false;

                match result {
                    Ok(mut data) => {
                        // Apply client-side system filtering
                        if self.exclude_system {
                            self.filter_system_processes(&mut data.tree);
                        }

                        let count = data.tree.nodes.len();
                        self.tree_data = Some(data.tree);
                        self.statistics = Some(data.statistics);
                        self.fit_view_trigger = true;

                        self.show_alert(
                            format!("Analysis complete: {} processes displayed", count),
                            AlertType::Success,
                        );
                    }
                    Err(err) => {
                        self.show_alert(err, AlertType::Error);
                    }
                }

                true
            }

            Msg::SelectNode(node_id) => {
                self.selected_node_id = node_id;
                true
            }

            Msg::Logout => {
                let api = self.api.clone();
                ctx.link().send_future(async move {
                    api.logout().await.ok();
                    if let Some(window) = window() {
                        window.location().set_href("/login").ok();
                    }
                    Msg::LoadClassifications // Dummy message
                });
                false
            }

            Msg::FitView => {
                self.fit_view_trigger = true;
                true
            }

            Msg::LoadClassifications => {
                let api = self.api.clone();
                ctx.link().send_future(async move {
                    let result = api.get_classifications().await;
                    Msg::ClassificationsLoaded(result)
                });
                false
            }

            Msg::ClassificationsLoaded(result) => {
                if let Ok(classifications) = result {
                    self.classifications = classifications;
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_light = self.theme == Theme::Light;
        let theme_class = if is_light { "light-mode" } else { "" };

        html! {
            <div class={classes!("app-container", theme_class)}>
                <Controls
                    theme={self.theme.clone()}
                    on_toggle_theme={ctx.link().callback(|_| Msg::ToggleTheme)}
                    on_fit_view={ctx.link().callback(|_| Msg::FitView)}
                    on_logout={ctx.link().callback(|_| Msg::Logout)}
                />

                <div class="main-layout">
                    {self.render_left_sidebar(ctx)}

                    <div class="graph-container">
                        {self.render_alert()}
                        {self.render_loading()}

                        <GraphView
                            tree_data={self.tree_data.clone()}
                            is_light_mode={is_light}
                            on_node_select={ctx.link().callback(Msg::SelectNode)}
                            fit_view_trigger={self.fit_view_trigger}
                        />
                    </div>

                    {self.render_right_sidebar(ctx)}
                </div>

                {self.render_sidebar_toggles(ctx)}
            </div>
        }
    }
}

impl App {
    fn render_left_sidebar(&self, ctx: &Context<Self>) -> Html {
        if !self.left_sidebar_visible {
            return html! {};
        }

        html! {
            <Sidebar side={SidebarSide::Left} on_close={ctx.link().callback(|_| Msg::ToggleSidebar(SidebarSide::Left))}>
                <div class="sidebar-section">
                    <h3>{"Analysis Parameters"}</h3>

                    <div class="form-group">
                        <label>{"Agent ID"}</label>
                        <input
                            type="text"
                            value={self.agent_id.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                Msg::SetAgentId(input.value())
                            })}
                            placeholder="Enter agent ID"
                        />
                    </div>

                    <div class="form-group">
                        <label>{"Start Time"}</label>
                        <input
                            type="datetime-local"
                            value={self.start_time.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                Msg::SetStartTime(input.value())
                            })}
                        />
                    </div>

                    <div class="form-group">
                        <label>{"End Time"}</label>
                        <input
                            type="datetime-local"
                            value={self.end_time.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                Msg::SetEndTime(input.value())
                            })}
                        />
                    </div>

                    <button
                        class="btn-primary"
                        onclick={ctx.link().callback(|_| Msg::Analyze)}
                        disabled={self.loading}
                    >
                        {"Analyze"}
                    </button>
                </div>

                <div class="sidebar-section">
                    <h3>{"Filters"}</h3>

                    <div class="form-group">
                        <label>{"Process Name"}</label>
                        <input
                            type="text"
                            value={self.filter_process.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                Msg::SetFilterProcess(input.value())
                            })}
                            placeholder="Filter by process name"
                        />
                    </div>

                    <div class="form-group">
                        <label>{"User"}</label>
                        <input
                            type="text"
                            value={self.filter_user.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                Msg::SetFilterUser(input.value())
                            })}
                            placeholder="Filter by user"
                        />
                    </div>

                    <div class="form-group">
                        <label>{"Command Contains"}</label>
                        <input
                            type="text"
                            value={self.filter_cmd.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                Msg::SetFilterCmd(input.value())
                            })}
                            placeholder="Search in command line"
                        />
                    </div>

                    <div class="form-group checkbox-group">
                        <label>
                            <input
                                type="checkbox"
                                checked={self.exclude_system}
                                onchange={ctx.link().callback(|e: Event| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::SetExcludeSystem(input.checked())
                                })}
                            />
                            {" Exclude System Processes"}
                        </label>
                    </div>
                </div>

                {self.render_statistics()}
            </Sidebar>
        }
    }

    fn render_right_sidebar(&self, ctx: &Context<Self>) -> Html {
        if !self.right_sidebar_visible {
            return html! {};
        }

        let selected_node = self.tree_data.as_ref().and_then(|tree| {
            self.selected_node_id.as_ref().and_then(|id| {
                tree.nodes.iter().find(|n| n.id == *id)
            })
        });

        html! {
            <Sidebar side={SidebarSide::Right} on_close={ctx.link().callback(|_| Msg::ToggleSidebar(SidebarSide::Right))}>
                {
                    if let Some(node) = selected_node {
                        html! {
                            <DetailsPanel node={node.clone()} />
                        }
                    } else {
                        html! {
                            <div class="sidebar-section">
                                <h3>{"Legend"}</h3>
                                {self.render_legend()}
                            </div>
                        }
                    }
                }
            </Sidebar>
        }
    }

    fn render_statistics(&self) -> Html {
        if let Some(stats) = &self.statistics {
            let total = self.tree_data.as_ref().map(|t| t.nodes.len()).unwrap_or(0);
            let orphans = self.tree_data.as_ref().map(|t| t.orphans.len()).unwrap_or(0);

            html! {
                <div class="sidebar-section">
                    <h3>{"Statistics"}</h3>
                    <div class="stats-grid">
                        <div class="stat-item">
                            <div class="stat-value">{total}</div>
                            <div class="stat-label">{"Total Processes"}</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value">{stats.suspicious_count}</div>
                            <div class="stat-label">{"Suspicious"}</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value">{orphans}</div>
                            <div class="stat-label">{"Orphans"}</div>
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn render_legend(&self) -> Html {
        if let Some(tree) = &self.tree_data {
            let mut classification_counts: std::collections::HashMap<String, (usize, String)> = std::collections::HashMap::new();

            for node in &tree.nodes {
                let entry = classification_counts
                    .entry(node.classification.clone())
                    .or_insert((0, node.color.background.clone()));
                entry.0 += 1;
            }

            let mut items: Vec<_> = classification_counts.into_iter().collect();
            items.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

            html! {
                <div class="legend-container">
                    {
                        items.iter().map(|(name, (count, color))| {
                            html! {
                                <div class="legend-item">
                                    <div class="legend-color" style={format!("background: {}", color)}></div>
                                    <div class="legend-info">
                                        <div class="legend-name">{name}</div>
                                        <div class="legend-count">{format!("{} processes", count)}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            }
        } else {
            html! {
                <p class="empty-message">{"No data to display"}</p>
            }
        }
    }

    fn render_sidebar_toggles(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                {
                    if !self.left_sidebar_visible {
                        html! {
                            <button
                                class="sidebar-toggle left"
                                onclick={ctx.link().callback(|_| Msg::ToggleSidebar(SidebarSide::Left))}
                            >
                                {"▶"}
                            </button>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if !self.right_sidebar_visible {
                        html! {
                            <button
                                class="sidebar-toggle right"
                                onclick={ctx.link().callback(|_| Msg::ToggleSidebar(SidebarSide::Right))}
                            >
                                {"◀"}
                            </button>
                        }
                    } else {
                        html! {}
                    }
                }
            </>
        }
    }

    fn render_alert(&self) -> Html {
        if let Some((message, alert_type)) = &self.alert_message {
            let class = match alert_type {
                AlertType::Success => "alert-success",
                AlertType::Error => "alert-error",
                AlertType::Info => "alert-info",
            };

            html! {
                <div class={classes!("alert", class)}>
                    {message}
                </div>
            }
        } else {
            html! {}
        }
    }

    fn render_loading(&self) -> Html {
        if self.loading {
            html! {
                <div class="loading-overlay">
                    <div class="loading-spinner"></div>
                    <div>{"Loading..."}</div>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn show_alert(&mut self, message: String, alert_type: AlertType) {
        self.alert_message = Some((message, alert_type));
        // Auto-hide after 5 seconds
        // We'd need to use a timeout here, but for simplicity we'll leave it visible
    }

    fn filter_system_processes(&self, tree: &mut ProcessTree) {
        let system_classifications = vec!["System Process".to_lowercase(), "system".to_lowercase()];

        let system_node_ids: std::collections::HashSet<_> = tree
            .nodes
            .iter()
            .filter(|node| {
                system_classifications.iter().any(|sc| {
                    node.classification.to_lowercase().contains(sc)
                })
            })
            .map(|node| node.id.clone())
            .collect();

        tree.nodes.retain(|node| !system_node_ids.contains(&node.id));
        tree.edges.retain(|edge| {
            !system_node_ids.contains(&edge.from) && !system_node_ids.contains(&edge.to)
        });
        tree.orphans.retain(|orphan| {
            !system_node_ids.contains(&orphan.process_id.to_string())
        });
        tree.total_processes = tree.nodes.len();
    }
}
