use crate::my_array::*;
use crate::my_point::*;

// An open map is not defined by cells with specific values. Instead your objects have just coordinates,
// which you have to manage. Therefore MyOpenMap does not contain data of elements on map, but provides
// you with general game data and some usefull tools for frequently occuring tasks in open map scenarios.
// Expand and adapt with usefull game data and functions as needed.

#[derive(Copy, Clone, Default)]
pub struct Entity {
    pos: Point,
}

impl Entity {
    fn _new(x: i32, y: i32) -> Self {
        Entity {
            pos: Point::new(x, y),
        }
    }
}

#[derive(Copy, Clone)]
pub struct MyOpenMap {
    // we keep it here simply and asume 0,0 as origin of max
    pub max_x: i32,
    pub max_y: i32,
    pub my_base_is_at_origin: bool,
    pub attack_range: f32,
    pub my_base: Point,
    pub enemy_base: Point,
    pub my_defense_outpost: Point,
    pub my_attack_outpost: Point,
}

impl MyOpenMap {
    pub fn new(max_x: i32, max_y: i32, my_base_is_at_origin: bool, attack_range: f32) -> MyOpenMap {
        let mut result = MyOpenMap {
            max_x,
            max_y,
            my_base_is_at_origin,
            attack_range,
            my_base: Point::new(0, 0),
            enemy_base: Point::new(0, 0),
            my_defense_outpost: Point::new(0, 0),
            my_attack_outpost: Point::new(0, 0),
        };
        // we expect bases to be in corners of map with one map at origin and the other at max_x and max_y
        if my_base_is_at_origin {
            result.my_base = Point::new(0, 0);
            result.enemy_base = Point::new(max_x, max_y);
        } else {
            result.my_base = Point::new(max_x, max_y);
            result.enemy_base = Point::new(0, 0);
        };
        result
    }
    pub fn get_middle_point_of_map(&self) -> Point {
        Point::new(self.max_x / 2, self.max_y / 2)
    }
    pub fn set_outposts(&mut self, vector_defense_outpost: Point, vector_attack_outpost: Point) {
        // vectors a given assuming:
        //    1.) my_base is at origin
        //    2.) defense vector points from my_base to my_defense_outpost
        //    3.) attack vector points from enemy_base to my_attack_outpost
        // this function handels my_base at max_x and max_y
        // therefore user of code does not have to think about inverting vectors, etc.
        if self.my_base_is_at_origin {
            self.my_defense_outpost = self.my_base.add(vector_defense_outpost);
            self.my_attack_outpost = self.enemy_base.add(vector_attack_outpost);
        } else {
            self.my_defense_outpost = self.my_base.subtract(vector_defense_outpost);
            self.my_attack_outpost = self.enemy_base.subtract(vector_attack_outpost);
        }
    }
    pub fn best_defense_position(&self, mut monsters: MyArray<Entity, 100>) -> Point {
        // monsters try attack the base. find best defense postion to attack as many monsters at the same time
        // and try to prevent monsters to enter your base (meaning attack the monsters nearest to base).

        // sort monsters by distance to base
        monsters.as_slice_mut().sort_by(|a, b| {
            a.pos
                .distance(self.my_base)
                .partial_cmp(&b.pos.distance(self.my_base))
                .unwrap()
        });
        let start_monster = match monsters.get(0) {
            Some(start_monster) => *start_monster,
            None => return self.my_defense_outpost,
        };

        // remove all monsters from list which are too far away from start_monster
        let mut index = 0;
        while index < monsters.len() {
            if monsters.get(index).unwrap().pos.distance(start_monster.pos)
                <= self.attack_range * 2.0
            {
                index += 1;
            } else {
                monsters.remove(index);
            }
        }

        let defense_point = self.calc_centroid(start_monster.pos, self.my_base, monsters);

        if start_monster.pos.distance(defense_point) <= self.attack_range {
            defense_point
        } else {
            // if centroid is to far away from start_point, move attack_range from start_monster to defense_point
            start_monster
                .pos
                .scale_toward_point_with_len(defense_point, self.attack_range)
        }
    }
    pub fn best_farming_position(
        &self,
        farmer: &Entity,
        mut monsters: MyArray<Entity, 100>,
    ) -> Point {
        // remove all monsters to far away from farmer
        let mut index = 0;
        while index < monsters.len() {
            if monsters.get(index).unwrap().pos.distance(farmer.pos) <= self.attack_range * 2.0 {
                index += 1;
            } else {
                monsters.remove(index);
            }
        }
        // sort monsters by distance to farmer in descending order
        monsters.as_slice_mut().sort_by(|a, b| {
            b.pos
                .distance(farmer.pos)
                .partial_cmp(&a.pos.distance(farmer.pos))
                .unwrap()
        });
        let mut farm_point = farmer.pos;
        let mut search_farm_point = true;
        while search_farm_point {
            // start_monster is monster farthest away from farmer to make sure, that it is on outline of polygon
            let start_monster = monsters.get(0).unwrap();
            farm_point = self.calc_centroid(
                start_monster.pos,
                farmer
                    .pos
                    .scale_toward_point_with_factor(start_monster.pos, 2.0),
                monsters,
            );
            let monsters_in_attack_range = monsters
                .iter()
                .filter(|m| m.pos.distance(farm_point) <= self.attack_range)
                .count();
            if monsters_in_attack_range < monsters.len() - 1 {
                // sort monsters by distance to farm point in descending order
                monsters.as_slice_mut().sort_by(|a, b| {
                    b.pos
                        .distance(farm_point)
                        .partial_cmp(&a.pos.distance(farm_point))
                        .unwrap()
                });
                monsters.remove(0);
            } else {
                search_farm_point = false;
            }
        }
        farm_point
    }
    pub fn calc_centroid(
        &self,
        start_point: Point,
        reference_point: Point,
        mut unsorted_polygon: MyArray<Entity, 100>,
    ) -> Point {
        // let's try to build a non-self-intersecting closed and sorted by angle polygon from a list of unsorted entities,
        // which centroid is probably the best defense position. See https://de.wikipedia.org/wiki/Geometrischer_Schwerpunkt#Polygon
        // Handels the following special cases: multiple entities on same point, entities on outline without a corner,
        // entities inside polygon, no polygon but multiple entities on same angle, just one or two entities
        // start_point must be on outline
        // reference point defines starting angle; must be outside of polygon
        let mut polygon: MyArray<Point, 100> = MyArray::new();
        let mut current_point = start_point;
        let mut last_point = reference_point;
        let mut not_origin = true;
        // fill polygon in order of angle
        while not_origin && unsorted_polygon.len() > 0 {
            let current_alpha = last_point.subtract(current_point).angle();
            // sort unsorted_polygon regarding to current_point and current_alpha
            unsorted_polygon.as_slice_mut().sort_by(|a, b| {
                ((a.pos.subtract(current_point).angle() - current_alpha + 360.0) % 360.0)
                    .partial_cmp(
                        &((b.pos.subtract(current_point).angle() - current_alpha + 360.0) % 360.0),
                    )
                    .unwrap()
            });
            // get next_point and filter any point on same position as current_point and last_point
            let next_point = unsorted_polygon
                .iter()
                .enumerate()
                .filter(|(_, m)| m.pos != current_point && m.pos != last_point)
                .next();
            if next_point.is_none() {
                return start_point; // this happens, if all points are on same position or just one point is available
            }
            let (m_index, _) = next_point.unwrap();
            let next_point = unsorted_polygon.remove(m_index).unwrap().pos;
            polygon.push(next_point);
            last_point = current_point;
            current_point = next_point;
            not_origin = next_point != start_point; // stop if next_point is start_point
        }

        if not_origin {
            // polygon did not close on start point -> return start_point
            return start_point;
        }

        // calc centroid of polygon
        // 1. calc area of polygon
        let mut polygon_area = 0;
        let mut last_point = start_point;
        for point in polygon.iter() {
            polygon_area += last_point.x * point.y - point.x * last_point.y;
            last_point = *point;
        }
        // devide by 2 and multiply by 6 equals multiply by 3
        polygon_area *= 3;

        if polygon_area == 0 {
            // multiple points are on one angle respectively line toward start_point
            // choose middle point between start_point and point farthest away
            polygon.as_slice_mut().sort_by(|a, b| {
                b.distance(start_point)
                    .partial_cmp(&a.distance(start_point))
                    .unwrap()
            });
            let mv = polygon.get(0).unwrap().subtract(start_point); // vector from start_point to point 0
            start_point.add(mv.scale(0.5)) // move to middle of both points
        } else {
            let mut x_centroid = 0;
            let mut last_point = start_point;
            for point in polygon.iter() {
                x_centroid +=
                    (last_point.x + point.x) * (last_point.x * point.y - point.x * last_point.y);
                last_point = *point;
            }
            let mut y_centroid = 0;
            let mut last_point = start_point;
            for point in polygon.iter() {
                y_centroid +=
                    (last_point.y + point.y) * (last_point.x * point.y - point.x * last_point.y);
                last_point = *point;
            }
            Point::new(x_centroid / polygon_area, y_centroid / polygon_area)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_defense_point() {
        let game_data = MyOpenMap::new(1000, 1000, true, 100.0);
        let mut monsters: MyArray<Entity, 100> = MyArray::new();
        // quadrat
        let monster_0 = Entity::_new(10, 10);
        let monster_1 = Entity::_new(20, 10);
        let monster_2 = Entity::_new(10, 20);
        let monster_3 = Entity::_new(20, 20);
        monsters.push(monster_0);
        monsters.push(monster_1);
        monsters.push(monster_2);
        monsters.push(monster_3);
        let defense_point = game_data.best_defense_position(monsters);
        assert_eq!(defense_point, Point::new(15, 15));
        // raute plus unneccescary extra points at the same position,  on curve, and inside raute
        monsters.flush();
        let monster_0 = Entity::_new(10, 10);
        let monster_1 = Entity::_new(20, 30);
        let monster_2 = Entity::_new(10, 20);
        let monster_3 = Entity::_new(20, 20);
        let monster_4 = Entity::_new(20, 25);
        let monster_5 = Entity::_new(14, 18);
        let monster_6 = Entity::_new(10, 20);
        monsters.push(monster_0);
        monsters.push(monster_1);
        monsters.push(monster_2);
        monsters.push(monster_3);
        monsters.push(monster_4);
        monsters.push(monster_5);
        monsters.push(monster_6);
        let defense_point = game_data.best_defense_position(monsters);
        assert_eq!(defense_point, Point::new(15, 20));
        // multiply monsters on one angle
        monsters.flush();
        let monster_0 = Entity::_new(10, 10);
        let monster_1 = Entity::_new(30, 30);
        let monster_2 = Entity::_new(15, 15);
        monsters.push(monster_0);
        monsters.push(monster_1);
        monsters.push(monster_2);
        let defense_point = game_data.best_defense_position(monsters);
        assert_eq!(defense_point, Point::new(20, 20));
        // multiply monsters on one point
        monsters.flush();
        let monster_0 = Entity::_new(10, 10);
        let monster_1 = Entity::_new(10, 10);
        let monster_2 = Entity::_new(10, 10);
        monsters.push(monster_0);
        monsters.push(monster_1);
        monsters.push(monster_2);
        let defense_point = game_data.best_defense_position(monsters);
        assert_eq!(defense_point, Point::new(10, 10));
        // just one monster and another one to far away
        monsters.flush();
        let monster_0 = Entity::_new(10, 10);
        let monster_1 = Entity::_new(500, 300);
        monsters.push(monster_0);
        monsters.push(monster_1);
        let defense_point = game_data.best_defense_position(monsters);
        assert_eq!(defense_point, Point::new(10, 10));
        // no monsters at all
        monsters.flush();
        let defense_point = game_data.best_defense_position(monsters);
        assert_eq!(defense_point, Point::new(0, 0));
    }

    #[test]
    fn test_farming_point() {
        let game_data = MyOpenMap::new(1000, 1000, true, 100.0);
        let mut monsters: MyArray<Entity, 100> = MyArray::new();
        let farmer = Entity::_new(
            game_data.get_middle_point_of_map().x,
            game_data.get_middle_point_of_map().y,
        );
        assert_eq!(farmer.pos, Point::new(500, 500));
        let monster_0 = Entity::_new(500, 450);
        let monster_1 = Entity::_new(20, 30);
        let monster_2 = Entity::_new(600, 550);
        let monster_3 = Entity::_new(900, 700);
        let monster_4 = Entity::_new(400, 550);
        let monster_5 = Entity::_new(500, 650);
        let monster_6 = Entity::_new(400, 450);
        monsters.push(monster_0);
        monsters.push(monster_1);
        monsters.push(monster_2);
        monsters.push(monster_3);
        monsters.push(monster_4);
        monsters.push(monster_5);
        monsters.push(monster_6);
        let attack_point = game_data.best_farming_position(&farmer, monsters);
        assert_eq!(attack_point, Point::new(500, 550));
    }
}
