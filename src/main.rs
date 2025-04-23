// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use std::io::Cursor;
use winit::window::Icon;

mod helpers;
mod object;

use helpers::camera_controller::{CameraController, CameraControllerPlugin};
use std::{
    fmt::{self, Formatter},
    vec,
};

use bevy::{
    color::palettes::{css::*, tailwind::*},
    ecs::system::IntoObserverSystem,
    input::common_conditions::input_just_pressed,
    math::vec3,
    prelude::*,
    render::{
        camera::{Exposure, PhysicalCameraParameters},
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
    text::FontSmoothing,
    window::WindowResolution,
};

fn main() {
    let mut app = App::new();

    let mut builder = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "QTT 110 米 主反射面 3D 模拟器".to_string(),
            resolution: WindowResolution::new(800., 600.),
            ..default()
        }),
        ..default()
    });

    app.add_plugins(builder);

    app.insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_systems(Startup, set_window_icon);

    app.add_plugins(MaterialPlugin::<CustomMaterial>::default())
        .add_plugins(CameraControllerPlugin)
        .insert_resource(Parameters(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 1000.0,
            sensor_height: 0.01866,
        }))
        .init_state::<MockingDataFn>()
        .init_state::<MockingState>()
        .init_state::<MockingInterpolateAlgo>()
        .init_state::<BoundaryRender>()
        .insert_resource(MockingSpeed(2.5))
        .add_systems(
            Startup,
            (setup, setup_instruction, setup_control_ui).chain(),
        )
        .add_systems(
            Update,
            (
                button_system,
                update_exposure,
                toggle_text_visibility.run_if(input_just_pressed(KeyCode::KeyH)),
                update.run_if(in_state(MockingState::Start)),
                // rotate_camera3d,
            ),
        );

    app.run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let Some(primary) = windows.get_window(primary_entity) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/reflector.wgsl";

/// The initial position of the camera.
const CAMERA_INITIAL_POSITION: Vec3 = vec3(0.0, 18.0, 0.0);

/// The initial position of the camera.
const CAMERA_INITIAL_POSITION_Z: Vec3 = vec3(0.0, 0.0, 25.0);

#[derive(Resource, Default, Deref, DerefMut)]
struct Parameters(PhysicalCameraParameters);

#[derive(Component)]
struct Block;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
enum MockingState {
    Start,
    #[default]
    Stop,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
enum MockingDataFn {
    #[default]
    Mock1 = 1,
    Mock2 = 2,
    Networking = 3,
}

impl fmt::Display for MockingDataFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MockingDataFn::Mock1 => write!(f, "刚性面"),
            MockingDataFn::Mock2 => write!(f, "柔性面"),
            MockingDataFn::Networking => write!(f, "Networking"),
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct MockingSpeed(f32);

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
enum MockingInterpolateAlgo {
    Interpolate_0 = 0,
    Interpolate_1 = 1,
    #[default]
    Interpolate_Normal = 2,
    Interpolate_Oklab = 3,
}

impl fmt::Display for MockingInterpolateAlgo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MockingInterpolateAlgo::Interpolate_0 => write!(f, "algo_0"),
            MockingInterpolateAlgo::Interpolate_1 => write!(f, "algo_1"),
            MockingInterpolateAlgo::Interpolate_Normal => write!(f, "algo_normal"),
            MockingInterpolateAlgo::Interpolate_Oklab => write!(f, "algo_oklab"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
enum BoundaryRender {
    #[default]
    Disable = 0,
    Enable = 1,
}

impl fmt::Display for BoundaryRender {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundaryRender::Disable => write!(f, "禁用"),
            BoundaryRender::Enable => write!(f, "启用"),
        }
    }
}

// Holds a handle to the custom material
#[derive(Resource)]
struct CustomMaterialHandle(Handle<CustomMaterial>);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    // 顶点传感器高度 GPU 缓冲 buffer
    #[storage(0, read_only)]
    buffer: Handle<ShaderStorageBuffer>,

    // 是否启用边界渲染
    #[uniform(1)]
    enable_boundary_render: u32,

    // 颜色算法选择
    #[uniform(2)]
    interpolate_algo: u32,
}

impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        // 禁用背面剔除
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    boundary_render: Res<State<BoundaryRender>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    // 加载自定义字体
    let font = asset_server.load("fonts/SIMYOU.TTF");
    let custom_font = CustomTextFont(font.clone());
    commands.insert_resource(custom_font);

    // add camera
    commands.spawn((
        Camera3d::default(),
        CameraController {
            enabled: false,
            ..default()
        },
        Transform::from_translation(CAMERA_INITIAL_POSITION).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    // Light
    commands.spawn(DirectionalLight::default());

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.03).mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));

    // add plane
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(100.0, 100.0)
                    .subdivisions(10),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: SILVER.into(),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(0.0, 0.0, -8.0)),
    ));

    // setup mesh
    let positions = &object::vertices;
    let mut index = 0; // position index
    let mut indices = vec![];
    // 创建顶点颜色数据
    let vertex_count = positions.len();
    let colors: Vec<[f32; 4]> = (0..vertex_count)
        .map(|i| {
            let t = i as f32 / vertex_count as f32;
            [t, 1.0 - t, 0.5, 1.0] // RGBA
        })
        .collect();

    let uv = (0..positions.len())
        .map(|i| match i % 4 {
            0 => [0.0, 0.0],
            1 => [1.0, 0.0],
            2 => [1.0, 1.0],
            _ => [0.0, 1.0],
        })
        .collect::<Vec<[f32; 2]>>();
    let normal = (0..positions.len())
        .map(|i| [0.0, 1.0, 0.0])
        .collect::<Vec<[f32; 3]>>();

    while index < positions.len() {
        // 计算索引
        let i0 = index;
        let i1 = index + 1;
        let i2 = index + 2;
        let i3 = index + 3;

        // 添加索引
        indices.push(i0 as u32);
        indices.push(i1 as u32);
        indices.push(i2 as u32);

        indices.push(i0 as u32);
        indices.push(i2 as u32);
        indices.push(i3 as u32);

        index += 4;
    }

    // buffer
    let buffer = buffers.add(ShaderStorageBuffer::from(
        (0..positions.len())
            .map(|i| match i % 4 {
                0 => 0.0,
                1 => 0.33,
                2 => 0.66,
                _ => 1.0,
            })
            .collect::<Vec<f32>>(),
    ));

    // 是否允许边界渲染
    let enable_boundary = match *boundary_render.get() {
        BoundaryRender::Enable => 1,
        BoundaryRender::Disable => 0,
    };

    // Create the custom material with the storage buffer
    let custom_material = CustomMaterial {
        buffer: buffer,
        enable_boundary_render: enable_boundary,
        interpolate_algo: MockingInterpolateAlgo::Interpolate_Normal as u32,
    };

    let material_handle = custom_materials.add(custom_material);
    commands.insert_resource(CustomMaterialHandle(material_handle.clone()));

    // 抛物面(网格顶点 + 面)
    commands
        .spawn((Node { ..default() }, Transform::from_xyz(0.0, 0.0, -3.0)))
        .with_children(|p| {
            // 顶点
            for i in 0..positions.len() {
                p.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.03).mesh().uv(4, 4))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: ORANGE.into(),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::from_array(positions[i])),
                ));
            }

            // 反射面
            let mesh = create_mesh(positions, indices, colors, uv, normal);
            p.spawn((
                Block,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material_handle.clone()),
            ));
        });
}

#[derive(Component, Clone, Copy)]
enum ButtonID {
    SwitchMockingState,
    SwitchMockingFn,
    SwitchMockingBoundary,
    SwitchHelp,
    SwitchSpeedDecrease,
    SwitchSpeedIncrease,
    SwitchSpeedReset,
    SwitchColorAlgo,
    SwitchCameraLeft,
    SwitchCameraRight,
    SwitchCameraUp,
    SwitchCameraDown,
    SwitchCameraResetZ,
    SwitchCameraResetY,
}

#[derive(Component)]
struct SpeedText;

fn setup_control_ui(
    mut commands: Commands,
    custom_font_handle: Res<CustomTextFont>,
    camera_control: Single<&CameraController>,
) {
    let text_font = TextFont {
        font: (&custom_font_handle.0).clone(),
        font_size: 18.0,
        font_smoothing: FontSmoothing::AntiAliased,
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Auto,
                justify_content: JustifyContent::FlexStart,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(5.0),
                row_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(2.5)),
                ..default()
            },
            BackgroundColor(SILVER.with_alpha(0.2).into()),
            InheritedVisibility::default(),
            Transform::from_xyz(0.0, 0.0, 100.0),
        ))
        .with_children(|p| {
            // 添加 切换模拟状态 按钮
            spawn_button(
                p,
                "切换模拟状态",
                text_font.clone(),
                ButtonID::SwitchMockingState,
                on_switch_mocking_state_clicked,
            );
            // 添加 切换模拟函数 按钮
            spawn_button(
                p,
                format!("Shader函数: {}", MockingDataFn::Mock1).as_str(),
                text_font.clone(),
                ButtonID::SwitchMockingFn,
                on_switch_mocking_fn_clicked,
            );

            // 添加 切换模拟函数 按钮
            spawn_button(
                p,
                format!("颜色算法: {}", MockingInterpolateAlgo::Interpolate_Normal).as_str(),
                text_font.clone(),
                ButtonID::SwitchColorAlgo,
                on_switch_mocking_interpolate_algo_clicked,
            );

            // 添加 切换模拟函数 按钮
            spawn_button(
                p,
                format!("块边界: {}", BoundaryRender::Disable).as_str(),
                text_font.clone(),
                ButtonID::SwitchMockingBoundary,
                on_switch_boundary_clicked,
            );

            // 添加模拟速度控制
            p.spawn((
                Node {
                    position_type: PositionType::Relative,
                    width: Val::Auto,
                    height: Val::Auto,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(SILVER.with_alpha(0.8).into()),
            ))
            .with_children(|p1| {
                // 添加 "速度+" 控制按钮
                spawn_button(
                    p1,
                    "速度 -",
                    text_font.clone(),
                    ButtonID::SwitchSpeedDecrease,
                    get_switch_speed_fn(ButtonID::SwitchSpeedDecrease),
                );
                p1.spawn((
                    Text::new("2.0"),
                    SpeedText,
                    TextFont {
                        font_size: 20.0,
                        font: text_font.font.clone(),
                        ..default()
                    },
                    TextColor(BLUE.into()),
                ));
                spawn_button(
                    p1,
                    "速度 +",
                    text_font.clone(),
                    ButtonID::SwitchSpeedDecrease,
                    get_switch_speed_fn(ButtonID::SwitchSpeedIncrease),
                );
                spawn_button(
                    p1,
                    "重置",
                    text_font.clone(),
                    ButtonID::SwitchSpeedReset,
                    get_switch_speed_fn(ButtonID::SwitchSpeedReset),
                );
            });

            // 添加 帮助 按钮
            spawn_button(
                p,
                format!(
                    "相机控制: {}",
                    match camera_control.enabled {
                        true => "启用",
                        false => "禁用",
                    }
                )
                .as_str(),
                text_font.clone(),
                ButtonID::SwitchHelp,
                on_switch_camera_control_clicked,
            );

            // 添加相机转动控制
            p.spawn((
                Node {
                    position_type: PositionType::Relative,
                    width: Val::Auto,
                    height: Val::Auto,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(SILVER.with_alpha(0.8).into()),
            ))
            .with_children(|p1| {
                // 添加 "左" 控制按钮
                spawn_button(
                    p1,
                    "左",
                    text_font.clone(),
                    ButtonID::SwitchCameraLeft,
                    get_switch_camera_orientation_fn(ButtonID::SwitchCameraLeft),
                );
                spawn_button(
                    p1,
                    "右",
                    text_font.clone(),
                    ButtonID::SwitchCameraRight,
                    get_switch_camera_orientation_fn(ButtonID::SwitchCameraRight),
                );
                spawn_button(
                    p1,
                    "上",
                    text_font.clone(),
                    ButtonID::SwitchCameraRight,
                    get_switch_camera_orientation_fn(ButtonID::SwitchCameraUp),
                );
                spawn_button(
                    p1,
                    "下",
                    text_font.clone(),
                    ButtonID::SwitchCameraLeft,
                    get_switch_camera_orientation_fn(ButtonID::SwitchCameraDown),
                );
                spawn_button(
                    p1,
                    "重置(Z)",
                    text_font.clone(),
                    ButtonID::SwitchSpeedReset,
                    get_switch_camera_orientation_fn(ButtonID::SwitchCameraResetZ),
                );
                spawn_button(
                    p1,
                    "重置(Y)",
                    text_font.clone(),
                    ButtonID::SwitchSpeedReset,
                    get_switch_camera_orientation_fn(ButtonID::SwitchCameraResetY),
                );
            });

            // 添加 帮助 按钮
            spawn_button(
                p,
                "帮助",
                text_font.clone(),
                ButtonID::SwitchHelp,
                on_switch_help_clicked,
            );
        });
}

const NORMAL_BUTTON: Srgba = BLUE_500;
const HOVERED_BUTTON: Srgba = RED_300;
const PRESSED_BUTTON: Srgba = GREEN_500;

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn spawn_button<E: Event, B: Bundle, M>(
    parent: &mut ChildBuilder<'_>,
    text: &str,
    text_font: TextFont,
    id: ButtonID,
    event_handler: impl IntoObserverSystem<E, B, M>,
) {
    parent
        .spawn((
            id,
            Button::default(),
            Node {
                max_height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(5.)),
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(3.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON.into()),
        ))
        // 添加按钮点击事件
        .observe(event_handler)
        .with_children(|p| {
            p.spawn((Text::new(text), text_font));
        });
}

fn on_switch_mocking_fn_clicked(
    trigger: Trigger<Pointer<Down>>,
    data_fn: Res<State<MockingDataFn>>,
    mut next_data_fn: ResMut<NextState<MockingDataFn>>,
    query: Query<&Children>,
    mut text_query: Query<&mut Text>,
) {
    if let Ok(children) = query.get(trigger.entity()) {
        if let Ok(mut text) = text_query.get_mut(children[0]) {
            *text = Text::new(format!("Shader 函数: {}", data_fn.get()));
            match data_fn.get() {
                MockingDataFn::Mock1 => {
                    next_data_fn.set(MockingDataFn::Mock2);
                }
                MockingDataFn::Mock2 => {
                    next_data_fn.set(MockingDataFn::Networking);
                }
                MockingDataFn::Networking => {
                    next_data_fn.set(MockingDataFn::Mock1);
                }
            }
        }
    }
}

fn on_switch_mocking_state_clicked(
    trigger: Trigger<Pointer<Down>>,
    mocking_state: Res<State<MockingState>>,
    mut next_mocking_state: ResMut<NextState<MockingState>>,
    query: Query<&Children>,
    mut text_query: Query<&mut Text>,
) {
    if let Ok(children) = query.get(trigger.entity()) {
        if let Ok(mut text) = text_query.get_mut(children[0]) {
            match mocking_state.get() {
                MockingState::Start => {
                    *text = Text::new("模拟状态: 停止");
                    next_mocking_state.set(MockingState::Stop);
                }
                MockingState::Stop => {
                    *text = Text::new("模拟状态: 开始");
                    next_mocking_state.set(MockingState::Start);
                }
            }
        }
    }
}

fn on_switch_mocking_interpolate_algo_clicked(
    trigger: Trigger<Pointer<Down>>,
    mocking_algo: Res<State<MockingInterpolateAlgo>>,
    mut next_mocking_algo: ResMut<NextState<MockingInterpolateAlgo>>,
    query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    custom_material_handle: Res<CustomMaterialHandle>,
) {
    let custom_material = custom_materials.get_mut(&custom_material_handle.0).unwrap();
    if let Ok(children) = query.get(trigger.entity()) {
        if let Ok(mut text) = text_query.get_mut(children[0]) {
            match mocking_algo.get() {
                MockingInterpolateAlgo::Interpolate_0 => {
                    *text = Text::new("颜色算法: Interpolate_1");
                    next_mocking_algo.set(MockingInterpolateAlgo::Interpolate_1);
                    custom_material.interpolate_algo = 1;
                }
                MockingInterpolateAlgo::Interpolate_1 => {
                    *text = Text::new("颜色算法: Interpolate_Normal");
                    next_mocking_algo.set(MockingInterpolateAlgo::Interpolate_Normal);
                    custom_material.interpolate_algo = 2;
                }
                MockingInterpolateAlgo::Interpolate_Normal => {
                    *text = Text::new("颜色算法: Interpolate_Oklab");
                    next_mocking_algo.set(MockingInterpolateAlgo::Interpolate_Oklab);
                    custom_material.interpolate_algo = 3;
                }
                MockingInterpolateAlgo::Interpolate_Oklab => {
                    *text = Text::new("颜色算法: Interpolate_0");
                    next_mocking_algo.set(MockingInterpolateAlgo::Interpolate_0);
                    custom_material.interpolate_algo = 0;
                }
            }
        }
    }
}

fn on_switch_boundary_clicked(
    trigger: Trigger<Pointer<Down>>,
    boundary_state: Res<State<BoundaryRender>>,
    mut next_boundary_state: ResMut<NextState<BoundaryRender>>,
    query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    custom_material_handle: Res<CustomMaterialHandle>,
) {
    // 禁用边界渲染
    let custom_material = custom_materials.get_mut(&custom_material_handle.0).unwrap();
    if let Ok(children) = query.get(trigger.entity()) {
        if let Ok(mut text) = text_query.get_mut(children[0]) {
            match *boundary_state.get() {
                BoundaryRender::Enable => {
                    *text = Text::new("块边界: 禁用");
                    next_boundary_state.set(BoundaryRender::Disable);
                    custom_material.enable_boundary_render = 0;
                }
                BoundaryRender::Disable => {
                    *text = Text::new("块边界: 启用");
                    next_boundary_state.set(BoundaryRender::Enable);
                    custom_material.enable_boundary_render = 1;
                }
            }
        }
    }
}

fn on_switch_help_clicked(
    trigger: Trigger<Pointer<Down>>,
    mut query: Query<&mut Visibility, With<InstructionText>>,
) {
    query.iter_mut().for_each(|mut visibility| {
        if *visibility == Visibility::Visible {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    });
}

fn on_switch_camera_control_clicked(
    trigger: Trigger<Pointer<Down>>,
    query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    mut camera_query: Query<&mut CameraController, With<Camera3d>>,
) {
    camera_query.iter_mut().for_each(|mut camera_controller| {
        if let Ok(children) = query.get(trigger.entity()) {
            if let Ok(mut text) = text_query.get_mut(children[0]) {
                *camera_controller = CameraController {
                    enabled: !camera_controller.enabled,
                    ..*camera_controller
                };
                match camera_controller.enabled {
                    true => {
                        *text = Text::new("相机控制: 启用");
                    }
                    false => {
                        *text = Text::new("相机控制: 禁用");
                    }
                }
            }
        }
    });
}

fn get_switch_speed_fn(
    typ: ButtonID,
) -> impl FnMut(
    Trigger<Pointer<Down>>,
    Query<&Children>,
    Query<&mut Text, With<SpeedText>>,
    ResMut<MockingSpeed>,
) {
    move |trigger: Trigger<Pointer<Down>>,
          query: Query<&Children>,
          mut text_query: Query<&mut Text, With<SpeedText>>,
          mut speed: ResMut<MockingSpeed>| {
        if let Ok(children) = query.get(trigger.entity()) {
            match typ {
                ButtonID::SwitchSpeedDecrease => {
                    speed.0 -= 0.5;
                }
                ButtonID::SwitchSpeedIncrease => {
                    speed.0 += 0.5;
                }
                ButtonID::SwitchSpeedReset => {
                    speed.0 = 2.5;
                }
                _ => {}
            };
            let mut text = text_query.single_mut();
            *text = Text::new(format!("{:.1}", speed.0));
        }
    }
}

fn get_switch_camera_orientation_fn(
    typ: ButtonID,
) -> impl FnMut(Trigger<Pointer<Down>>, Single<&mut Transform, With<Camera>>) {
    move |trigger: Trigger<Pointer<Down>>, mut camera: Single<&mut Transform, With<Camera>>| {
        match typ {
            ButtonID::SwitchCameraLeft => {
                // Rotate camera left
                camera.rotate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Z, -0.1));
            }
            ButtonID::SwitchCameraRight => {
                // Rotate camera right
                camera.rotate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Z, 0.1));
            }
            ButtonID::SwitchCameraUp => {
                // Rotate camera up
                camera.rotate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, 0.1));
            }
            ButtonID::SwitchCameraDown => {
                // Rotate camera down
                camera.rotate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, -0.1));
            }
            ButtonID::SwitchCameraResetZ => {
                // Reset camera position
                camera.translation = CAMERA_INITIAL_POSITION;
                camera.look_at(Vec3::ZERO, Vec3::Z);
            }
            ButtonID::SwitchCameraResetY => {
                // Reset camera position
                camera.translation = CAMERA_INITIAL_POSITION_Z;
                camera.look_at(Vec3::ZERO, Vec3::Z);
            }
            _ => {}
        }
    }
}

#[derive(Component)]
struct InstructionText;

#[derive(Resource, Clone)]
struct CustomTextFont(pub Handle<Font>);

fn setup_instruction(
    mut commands: Commands,
    parameters: Res<Parameters>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/SIMYOU.TTF");
    let custom_font = CustomTextFont(font.clone());
    commands.insert_resource(custom_font);

    let text_font = TextFont {
        font: font.clone(),
        font_size: 24.0,
        font_smoothing: FontSmoothing::AntiAliased,
    };

    commands.spawn((
        Text::new("QTT 110 米 主反射面3D模拟 -- 高度参数"),
        TextFont {
            font: font.clone(),
            font_size: 48.0,
            font_smoothing: FontSmoothing::AntiAliased,
        },
        TextColor(BLUE_300.into()),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(50.0),
                ..default()
            },
            InstructionText,
            Visibility::Hidden,
            Text::default(),
        ))
        .with_children(|p| {
            p.spawn((
                TextSpan(format!("光圈: f/{:.0}\n", parameters.aperture_f_stops,)),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    font_smoothing: FontSmoothing::AntiAliased,
                },
                TextColor(RED.into()),
            ));
            p.spawn((
                TextColor(RED.into()),
                TextSpan(format!(
                    "快门速度: 1/{:.0}s\n",
                    1.0 / parameters.shutter_speed_s
                )),
                text_font.clone(),
            ));
            p.spawn((
                TextColor(RED.into()),
                TextSpan(format!("感光度: ISO {:.0}\n", parameters.sensitivity_iso)),
                text_font.clone(),
            ));
            p.spawn((
                TextSpan::new("\n\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("控制\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("---------------\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("1/2 - 减小/增加光圈\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("3/4 - 减小/增加快门速度\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("5/6 - 减小/增加感光度\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("R - 重置曝光\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("---------------\n"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new(
                    "相机控制:
鼠标\t- 移动相机方向
滚轮\t- 调整移动速度
Left\t- 按住以抓取光标
KeyM\t- 切换光标抓取
W/S/A/D- 上下左右飞移 
ShiftLeft\t- 按住时飞得更快",
                ),
                text_font.clone(),
                TextColor(RED.into()),
            ));
            p.spawn((
                TextSpan::new("\n---------------\nH 键显示帮助"),
                text_font.clone(),
                TextColor(RED.into()),
            ));
        });
}

fn update_exposure(
    key_input: Res<ButtonInput<KeyCode>>,
    mut parameters: ResMut<Parameters>,
    mut exposure: Single<&mut Exposure>,
    text: Single<Entity, (With<Text>, With<InstructionText>)>,
    mut writer: TextUiWriter,
) {
    // TODO: Clamp values to a reasonable range
    let entity = *text;
    if key_input.just_pressed(KeyCode::Digit2) {
        parameters.aperture_f_stops *= 2.0;
    } else if key_input.just_pressed(KeyCode::Digit1) {
        parameters.aperture_f_stops *= 0.5;
    }
    if key_input.just_pressed(KeyCode::Digit4) {
        parameters.shutter_speed_s *= 2.0;
    } else if key_input.just_pressed(KeyCode::Digit3) {
        parameters.shutter_speed_s *= 0.5;
    }
    if key_input.just_pressed(KeyCode::Digit6) {
        parameters.sensitivity_iso += 100.0;
    } else if key_input.just_pressed(KeyCode::Digit5) {
        parameters.sensitivity_iso -= 100.0;
    }
    if key_input.just_pressed(KeyCode::KeyR) {
        *parameters = Parameters::default();
    }

    *writer.text(entity, 1) = format!("光圈: f/{:.0}\n", parameters.aperture_f_stops);
    *writer.text(entity, 2) = format!("快门速度: 1/{:.0}s\n", 1.0 / parameters.shutter_speed_s);
    *writer.text(entity, 3) = format!("感光度: ISO {:.0}\n", parameters.sensitivity_iso);

    **exposure = Exposure::from_physical_camera(**parameters);
}

fn create_mesh(
    positions: &[[f32; 3]],
    indices: Vec<u32>,
    colors: Vec<[f32; 4]>,
    uv: Vec<[f32; 2]>,
    normal: Vec<[f32; 3]>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normal)
    .with_inserted_indices(Indices::U32(indices))
}

fn toggle_text_visibility(mut query: Query<&mut Visibility, With<InstructionText>>) {
    query.iter_mut().for_each(|mut visibility| {
        if *visibility == Visibility::Visible {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    });
}

fn update(
    time: Res<Time>,
    material_handle: Res<CustomMaterialHandle>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    data_fn: Res<State<MockingDataFn>>,
    speed: Res<MockingSpeed>,
) {
    let material = materials.get_mut(&material_handle.0).unwrap();
    let buffer = buffers.get_mut(&material.buffer).unwrap();
    let t = time.elapsed_secs() * speed.0;
    buffer.set_data(
        (0..3880)
            .map(|i| match data_fn.get() {
                MockingDataFn::Mock1 => mock1(t, i),
                MockingDataFn::Mock2 => mock2(t, i),
                MockingDataFn::Networking => mock1(t, i), // TODO: replace with networking data
            })
            .collect::<Vec<f32>>(),
    );
}

fn rotate_camera3d(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Camera3d)>,
    camera_control: Query<&CameraController>,
) {
    if camera_control.single().enabled {
        return;
    }
    let t = time.elapsed_secs() * 0.05;
    for (mut transform, _) in query.iter_mut() {
        *transform =
            Transform::from_translation(Quat::from_rotation_z(t).mul_vec3(CAMERA_INITIAL_POSITION))
                .looking_at(Vec3::ZERO, Vec3::Z);
    }
}

fn switch_mocking(current: Res<State<MockingState>>, mut next: ResMut<NextState<MockingState>>) {
    let next_state = match current.get() {
        MockingState::Start => MockingState::Stop,
        MockingState::Stop => MockingState::Start,
    };
    next.set(next_state);
}

// 模拟 1
fn mock1(t: f32, i: i32) -> f32 {
    let v = ops::sin(t + i as f32) * 0.5 + 0.5;
    match i % 4 {
        0 => v - 0.5,
        1 => v - 0.3,
        2 => v - 0.4,
        _ => v,
    }
}

fn mock2(t: f32, i: i32) -> f32 {
    let v1 = ops::sin(t + i as f32);
    let v2 = ops::sin(-t + i as f32 + 2.0);
    let v3 = ops::cos(t + i as f32 + 4.0);
    let v = (v1 + v2 + v3) / 3.0;
    match i % 4 {
        0 => v1,
        1 => v2,
        2 => v3,
        _ => v,
    }
}
