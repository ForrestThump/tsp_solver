use dashmap::DashMap;
use rayon::prelude::*;

use crate::point::Point;
use crate::point::Points;

#[derive(Clone)]
pub struct DistanceMap {
    pub map: DashMap<(u32, u32), f64>,
    pub num_points: u32,
}

impl DistanceMap {
    /* Calculate the points and store in distance map, then return map */
    pub fn new(points: &Points) -> DistanceMap {
        let num_points = points.points.len() as u32;
        let map = DashMap::with_capacity(num_points as usize * (num_points as usize - 1) / 2);

        points.points.par_iter().enumerate().for_each(|(i, point1)| {
            for (j, point2) in points.points.iter().enumerate().skip(i + 1) {
                let distance = DistanceMap::get_distance(point1, point2);
                map.insert((i as u32, j as u32), distance);
            }
        });

        DistanceMap { map, num_points }
    }
    
    pub fn point_count(&self) -> usize {
        self.num_points as usize
    }

    pub fn len(&self) -> usize {
        self.point_count()
    }

    pub fn get_distance_from_points(&self, point1: &u32, point2: &u32) ->f64 {
        /* Return 0 if they are the same point */
        if *point1 == *point2 { return 0.0; }

        /* Find the smaller point. */
        let (smaller, larger) = if point1 < point2 {
            (point1, point2)
        } else {
            (point2, point1)
        };
    
        /* Return the precomputed distance. */
        self.map.get(&(*smaller, *larger)).map_or(0.0, |v| *v)
    }

    fn get_distance(point1: &Point, point2: &Point) -> f64 {

        /* Emperical testing with a random, even distribution of points shows that NOT taking the square root
        * yields a 1% slower distance and greedy computation time compared to taking the square root. Evidently, 
        * the square root is not that expensive, and the increased variable size of not taking the square root is 
        * comparatively more expensive.
        *******************************/
        (f64::powf(point1.x-point2.x,2.0) + f64::powf(point1.y-point2.y,2.0)).sqrt()
    }    
}