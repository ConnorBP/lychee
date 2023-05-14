use vbsp::{Bsp,Brush, BrushFlags};
use crate::datatypes::tmp_vec3;

const DIST_EPSILON:f32  = 0.03125;

#[repr(C)]
struct trace_t {
    all_solid: bool,
    start_solid: bool,
    fraction: f32,
    fraction_left_solid: f32,

    end_pos: tmp_vec3,
    contents: u32,
    brush: Brush,
    num_brush_sides: u32

}

impl trace_t {
    fn clear(&mut self) {
        self.all_solid = true;
        self.start_solid = true;
        self.fraction = 1.0;
        self.fraction_left_solid= 1.0;
        self.end_pos = tmp_vec3{x:0.0,y:0.0,z:0.0};
        self.contents = 0;
        self.brush= Brush{ brush_side: 0, num_brush_sides: 0, flags: BrushFlags::EMPTY };
        self.num_brush_sides = 0;
    }
}

pub fn is_visible(
    map: &Bsp,
    origin: &tmp_vec3,
    destination: &tmp_vec3
) -> (bool,tmp_vec3)
{
    let mut trace = trace_t {
        all_solid: true,
        start_solid: true,
        fraction: 1.0,
        fraction_left_solid: 1.0,
        end_pos: tmp_vec3{x:0.0,y:0.0,z:0.0},
        contents: 0,
        brush: Brush{ brush_side: 0, num_brush_sides: 0, flags: BrushFlags::EMPTY },
        num_brush_sides: 0,
    };

    trace_ray( map, origin, destination, &mut trace );

    let res = !( trace.fraction < 0.98 );

    // if(!res) {
    //     println!("trace fraction {} was less than 1", trace.fraction);
    // }

    return (res, trace.end_pos);
}

fn trace_ray(
    map: &Bsp,
    origin: &tmp_vec3,
    finalvec: &tmp_vec3,
    out: &mut trace_t
)
{
    if !map.planes.is_empty() {

        out.clear();
        out.fraction = 1.0;
        out.fraction_left_solid = 0.0;

        ray_cast_node( map, 0, 0.0, 1.0, origin, finalvec, out );

        if out.fraction < 1.0 {
            for i in 0..2 {
                out.end_pos[ i ] = origin[i] + out.fraction * ( finalvec[i] - origin[i] );
            }
        }
        else {
            out.end_pos = *finalvec;
        }
    }
}

fn ray_cast_node(
    map: &Bsp,
    node_index: i32,
    start_fraction: f32,
    end_fraction: f32,
    origin: &tmp_vec3,
    destination: &tmp_vec3,
    out: &mut trace_t
)
{
    if out.fraction <= start_fraction  {
        return;
    }

    if node_index < 0  {
        let leaf = map.leaf(  -node_index as usize - 1 ).unwrap();
        for i in 0..leaf.leaf_brush_count-1 {

            let brush_index =  map.leaf_brushes.get( leaf.first_leaf_brush as usize + i as usize );
            if brush_index.is_none() {continue;}
            let brush_index = brush_index.unwrap();
            if let Some(brush) = map.brushes.get( brush_index.brush as usize ) {
                let  MASK_SHOT_HULL: BrushFlags  = BrushFlags::SOLID | BrushFlags::MOVEABLE | BrushFlags::MONSTER | BrushFlags::WINDOW | BrushFlags::DEBRIS | BrushFlags::GRATE;
                if (brush.flags & MASK_SHOT_HULL).bits() == 0  {
                    continue;
                }

                ray_cast_brush( map, brush, &origin, &destination, out );
                if out.fraction == 0.0  {
                    return;
                }

                out.brush = brush.clone();
            } else {
                continue;
            }

            

            
        }
        if out.start_solid || out.fraction < 1.0 {
            return;
        }
        for i in 0..leaf.leaf_face_count-1 {
            ray_cast_surface(map, map.leaf_faces.get( (leaf.first_leaf_face + i) as usize ).unwrap().face as usize, &origin, &destination, out );
        }
        return;
    }

    let node = map.node( node_index as usize );
    if node.is_none()  {
        return;
    }
    let node = node.unwrap();

    let plane = node.plane();

    let start_distance;
    let end_distance;

    if plane.ty < 3  {
        start_distance = origin[  plane.ty as usize  ] - plane.dist;
        end_distance   = destination[plane.ty as usize] - plane.dist;
    }
    else {
        start_distance = origin.dot( plane.normal.into() ) - plane.dist;
        end_distance = destination.dot( plane.normal.into() ) - plane.dist;
    }

    if start_distance >= 0.0 && end_distance >= 0.0  {
        ray_cast_node( map, node.children[0], start_fraction, end_fraction, origin, destination, out );
    }
    else if start_distance < 0.0 && end_distance < 0.0  {
        ray_cast_node( map, node.children[1], start_fraction, end_fraction, origin, destination, out );
    }
    else {
        let side_id;
        let mut fraction_first;
        let mut fraction_second;
        let mut middle: tmp_vec3 = tmp_vec3 {x:0.0,y:0.0,z:0.0};

        if start_distance < end_distance {
            // Back
            side_id = 1;
            let inversed_distance = 1.0 / ( start_distance - end_distance );

            fraction_first = ( start_distance + f32::EPSILON ) * inversed_distance; 
            fraction_second = ( start_distance + f32::EPSILON ) * inversed_distance;
        }
        else if end_distance < start_distance {
            // Front
            side_id = 0;
            let inversed_distance = 1.0 / ( start_distance - end_distance );

            fraction_first = ( start_distance + f32::EPSILON ) * inversed_distance;
            fraction_second = ( start_distance - f32::EPSILON ) * inversed_distance;
        }
        else {
            // Front
            side_id = 0;
            fraction_first = 1.0;
            fraction_second = 0.0;
        }
        if fraction_first < 0.0 {
            fraction_first = 0.0;
        }
        else if fraction_first > 1.0 {
            fraction_first = 1.0;
        }
        if fraction_second < 0.0 {
            fraction_second = 0.0;
        }
        else if fraction_second > 1.0 {
            fraction_second = 1.0;
        }

        let mut fraction_middle = start_fraction + ( end_fraction - start_fraction ) * fraction_first;
        for i in 0..2 {
            middle [i] = origin [i] + fraction_first * ( destination [i] - origin[i] );
        }

        ray_cast_node( map, node.children[side_id], start_fraction, fraction_middle, origin, &middle, out );
        fraction_middle = start_fraction + ( end_fraction - start_fraction ) * fraction_second;
        for i in 0..2 {
            middle[i] = origin[ i ] + fraction_second * ( destination[ i ] - origin [i] );
        }

        ray_cast_node( map, node.children[if side_id == 0 {1}else{0}], fraction_middle, end_fraction, &middle, destination, out );
    }
}

fn ray_cast_brush(
    map: &Bsp,
    brush: &Brush,
    origin: &tmp_vec3,
    destination: &tmp_vec3,
    out: &mut trace_t
)
{
    if  brush.num_brush_sides > 0 {
        let mut fraction_to_enter = -99.0;
        let mut fraction_to_leave = 1.0;
        let mut starts_out = false;
        let mut ends_out = false;
        for i in 0..brush.num_brush_sides-1  {
            let brush_side = map.brush_sides.get( (brush.brush_side + i) as usize );
            if brush_side.is_none() {
                continue;
            }
            let brush_side = brush_side.unwrap();

            let plane = map.planes.get( brush_side.plane as usize );
            if( plane.is_none() ) {
                continue;
            }
            let plane = plane.unwrap();

            let start_distance = origin.dot( plane.normal.into() ) - plane.dist;
            let end_distance = destination.dot( plane.normal.into() ) - plane.dist;
            if start_distance > 0.0  {
                starts_out = true;
                if end_distance > 0.0  {
                    return;
                }
            }
            else {
                if end_distance <= 0.0  {
                    continue;
                }
                ends_out = true;
            }
            if start_distance > end_distance  {
                let mut fraction = f32::max( start_distance - DIST_EPSILON , 0.0 );
                fraction = fraction / ( start_distance - end_distance );
                if( fraction > fraction_to_enter ) {
                    fraction_to_enter = fraction;
                }
            }
            else {
                let fraction = ( start_distance + DIST_EPSILON ) / ( start_distance - end_distance );
                if fraction < fraction_to_leave {
                    fraction_to_leave = fraction;
                }
            }
        }

        if starts_out  {
            if out.fraction_left_solid - fraction_to_enter > 0.0  {
                starts_out = false;
            }
        }

        out.num_brush_sides = brush.num_brush_sides;

        if( !starts_out ) {
            out.start_solid = true;
            out.contents = brush.flags.bits();

            if !ends_out {
                out.all_solid = true;
                out.fraction = 0.0;
                out.fraction_left_solid = 1.0;
            }
            else {
                if fraction_to_leave != 1.0 && fraction_to_leave > out.fraction_left_solid {
                    out.fraction_left_solid = fraction_to_leave;
                    if out.fraction <= fraction_to_leave {
                        out.fraction = 1.0;
                    }
                }
            }
            return;
        }

        if fraction_to_enter < fraction_to_leave  {
            if fraction_to_enter > -99.0 && fraction_to_enter < out.fraction  {
                if fraction_to_enter < 0.0  {
                    fraction_to_enter = 0.0;
                }

                out.fraction = fraction_to_enter;
                out.brush    = brush.clone();
                out.contents = brush.flags.bits();
            }
        }
    }
}

fn ray_cast_surface(
    map: &Bsp,
    surface_index: usize,
    origin: &tmp_vec3,
    destination: &tmp_vec3,
    out: &mut trace_t,
)
{
    // let index = surface_index ;
    // if index >= map.polygons.len()  {
    //     return;
    // }


    // auto* polygon   = &polygons.at( index );
    // auto* plane     = &polygon->plane;
    // const auto dot1 = plane->dist( origin );
    // const auto dot2 = plane->dist( destination );

    // if( dot1 > 0.f != dot2 > 0.f ) {
    //     if( dot1 - dot2 < valve::DIST_EPSILON ) {
    //         return;
    //     }

    //     const auto t = dot1 / ( dot1 - dot2 );
    //     if( t <= 0 ) {
    //         return;
    //     }

    //     std::size_t i = 0;
    //     const auto intersection = origin + ( destination - origin ) * t;
    //     for( ; i < polygon->num_verts; ++i ) {
    //         auto* edge_plane = &polygon->edge_planes.at( i );
    //         if( edge_plane->origin.is_zero() ) {
    //             edge_plane->origin = plane->origin - ( polygon->verts.at( i ) - polygon->verts.at( ( i + 1 ) % polygon->num_verts ) );
    //             edge_plane->origin.normalize();
    //             edge_plane->distance = edge_plane->origin.dot( polygon->verts.at( i ) );
    //         }
    //         if( edge_plane->dist( intersection ) < 0.0f ) {
    //             break;
    //         }
    //     }
    //     if( i == polygon->num_verts ) {
    //         out->fraction = 0.2f;
    //         out->end_pos = intersection;
    //     }
    // }
}