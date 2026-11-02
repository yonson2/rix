//! Filled and stroked 2D shapes drawn as meshes, tessellated with lyon ([013], D2).
//!
//! A [`Shape`] holds an outline and its colors. When it changes, [`plugin`] builds
//! one mesh with vertex colors (fill first, stroke on top) and draws it with a shared
//! white `ColorMaterial`.

use bevy::{
    asset::RenderAssetUsages,
    color::ColorToComponents,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use lyon_algorithms::rounded_polygon::add_rounded_polygon;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
    geom::{Box2D, point},
    path::{NO_ATTRIBUTES, Path, Polygon, Winding, builder::BorderRadii, math::Point},
};

pub(super) fn plugin(app: &mut App) {
    let material = app
        .world_mut()
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::default());
    app.insert_resource(ShapeMaterial(material))
        .add_systems(PostUpdate, mesh_shapes);
}

#[derive(Resource)]
struct ShapeMaterial(Handle<ColorMaterial>);

#[derive(Component, Clone)]
#[require(Transform, Visibility)]
pub struct Shape {
    pub path: Path,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
}

#[derive(Clone, Copy)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

impl Shape {
    pub fn new(path: Path) -> Self {
        Self {
            path,
            fill: None,
            stroke: None,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.stroke = Some(Stroke { color, width });
        self
    }

    pub fn mesh(&self) -> Mesh {
        let mut buffers: VertexBuffers<([f32; 2], [f32; 4]), u32> = VertexBuffers::new();
        if let Some(color) = self.fill {
            let color = color.to_linear().to_f32_array();
            if let Err(e) = FillTessellator::new().tessellate_path(
                &self.path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| {
                    (v.position().to_array(), color)
                }),
            ) {
                error!("fill tessellation failed: {e:?}");
            }
        }
        if let Some(stroke) = self.stroke {
            let color = stroke.color.to_linear().to_f32_array();
            if let Err(e) = StrokeTessellator::new().tessellate_path(
                &self.path,
                &StrokeOptions::default().with_line_width(stroke.width),
                &mut BuffersBuilder::new(&mut buffers, |v: StrokeVertex| {
                    (v.position().to_array(), color)
                }),
            ) {
                error!("stroke tessellation failed: {e:?}");
            }
        }

        let (positions, colors): (Vec<[f32; 3]>, Vec<[f32; 4]>) = buffers
            .vertices
            .into_iter()
            .map(|([x, y], color)| ([x, y, 0.], color))
            .unzip();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(buffers.indices))
    }
}

/// A closed polygon through `points`.
pub fn polygon(points: &[Vec2]) -> Path {
    let points = to_points(points);
    let mut builder = Path::builder();
    builder.add_polygon(Polygon {
        points: &points,
        closed: true,
    });
    builder.build()
}

/// A closed polygon through `points` with its corners rounded by `radius`.
pub fn rounded_polygon(points: &[Vec2], radius: f32) -> Path {
    let points = to_points(points);
    let mut builder = Path::builder();
    add_rounded_polygon(
        &mut builder,
        Polygon {
            points: &points,
            closed: true,
        },
        radius,
        NO_ATTRIBUTES,
    );
    builder.build()
}

/// A rectangle of `size` centred on the origin, with corners rounded by `radius`.
pub fn rounded_rectangle(size: Vec2, radius: f32) -> Path {
    let half = size / 2.;
    let mut builder = Path::builder();
    builder.add_rounded_rectangle(
        &Box2D::new(point(-half.x, -half.y), point(half.x, half.y)),
        &BorderRadii::new(radius),
        Winding::Positive,
    );
    builder.build()
}

fn to_points(points: &[Vec2]) -> Vec<Point> {
    points.iter().map(|p| point(p.x, p.y)).collect()
}

fn mesh_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ShapeMaterial>,
    shapes: Query<(Entity, &Shape), Changed<Shape>>,
) {
    for (entity, shape) in &shapes {
        commands.entity(entity).insert((
            Mesh2d(meshes.add(shape.mesh())),
            MeshMaterial2d(material.0.clone()),
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;

    use super::*;

    const SQUARE: [Vec2; 4] = [
        Vec2::new(0., 0.),
        Vec2::new(10., 0.),
        Vec2::new(10., 10.),
        Vec2::new(0., 10.),
    ];

    fn colors(mesh: &Mesh) -> Vec<[f32; 4]> {
        match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(colors)) => colors.clone(),
            _ => panic!("no vertex colors"),
        }
    }

    #[test]
    fn filled_square_is_two_triangles_in_its_color() {
        let mesh = Shape::new(polygon(&SQUARE)).fill(Color::WHITE).mesh();
        assert_eq!(mesh.count_vertices(), 4);
        assert_eq!(mesh.indices().map(|i| i.len()), Some(6));
        assert!(colors(&mesh).iter().all(|c| *c == [1., 1., 1., 1.]));
    }

    #[test]
    fn stroke_comes_after_the_fill() {
        let black = Color::BLACK.to_linear().to_f32_array();
        let mesh = Shape::new(polygon(&SQUARE))
            .fill(Color::WHITE)
            .stroke(Color::BLACK, 2.)
            .mesh();
        let colors = colors(&mesh);
        assert!(colors.len() > 4);
        assert!(colors[4..].iter().all(|c| *c == black));
    }

    #[test]
    fn empty_shape_is_an_empty_mesh() {
        assert_eq!(Shape::new(polygon(&SQUARE)).mesh().count_vertices(), 0);
    }

    #[test]
    fn rounded_shapes_stay_inside_their_corners() {
        let rect = rounded_rectangle(Vec2::new(20., 10.), 3.);
        let arrow = rounded_polygon(&SQUARE, 2.);
        for (path, min, max) in [
            (rect, Vec2::new(-10., -5.), Vec2::new(10., 5.)),
            (arrow, Vec2::ZERO, Vec2::splat(10.)),
        ] {
            let mesh = Shape::new(path).fill(Color::WHITE).mesh();
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("no positions");
            };
            assert!(positions.len() > 4);
            for [x, y, _] in positions {
                assert!((min.x - 1e-3..=max.x + 1e-3).contains(x), "x {x}");
                assert!((min.y - 1e-3..=max.y + 1e-3).contains(y), "y {y}");
            }
        }
    }
}
