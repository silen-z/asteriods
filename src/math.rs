use bevy::math::Vec2;

pub fn ray_circle_intersection(start: Vec2, dir: Vec2, origin: Vec2, radius: f32) -> Option<Vec2> {
    let l = -(start - origin);
    let tca = l.dot(dir);
    if tca < 0. {
        return None;
    }
    let d2 = l.dot(l) - tca * tca;
    if d2 > radius * radius {
        return None;
    }
    let thc = (radius * radius - d2).sqrt();
    Some(start + dir * (tca - thc))
}
