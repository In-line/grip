/*
 * gRIP
 * Copyright (c) 2018 Alik Aslanyan <cplusplus256@gmail.com>
 *
 *
 *    This program is free software; you can redistribute it and/or modify it
 *    under the terms of the GNU General Public License as published by the
 *    Free Software Foundation; either version 3 of the License, or (at
 *    your option) any later version.
 *
 *    This program is distributed in the hope that it will be useful, but
 *    WITHOUT ANY WARRANTY; without even the implied warranty of
 *    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 *    General Public License for more details.
 *
 *    You should have received a copy of the GNU General Public License
 *    along with this program; if not, write to the Free Software Foundation,
 *    Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 *    In addition, as a special exception, the author gives permission to
 *    link the code of this program with the Half-Life Game Engine ("HL
 *    Engine") and Modified Game Libraries ("MODs") developed by Valve,
 *    L.L.C ("Valve").  You must obey the GNU General Public License in all
 *    respects for all of the code used other than the HL Engine and MODs
 *    from Valve.  If you modify this file, you may extend this exception
 *    to your version of the file, but you are not obligated to do so.  If
 *    you do not wish to do so, delete this exception statement from your
 *    version.
 *
 */

use std::collections::HashMap;

/// Just a simple, general and limited abstraction for storing Pawn handle id's
pub struct CellMap<T> {
    inner: HashMap<isize, T>,
    counter: isize,
}

impl<T> Default for CellMap<T> {
    fn default() -> Self {
        CellMap::new()
    }
}
impl<T> CellMap<T> {
    pub fn new() -> CellMap<T> {
        CellMap {
            inner: HashMap::new(),
            counter: 1,
        }
    }

    /// Inserts desired item and returns generated id which is always greater than 1
    pub fn insert_with_unique_id(&mut self, item: T) -> isize {
        assert!(self.counter >= 1);

        assert!(self.inner.insert(self.counter as isize, item).is_none());
        self.counter += 1;

        self.counter - 1
    }

    pub fn remove_with_id(&mut self, id: isize) -> Option<T> {
        self.inner.remove(&id)
    }

    pub fn get_with_id(&self, id: isize) -> Option<&T> {
        self.inner.get(&id)
    }

    pub fn get_mut_with_id(&mut self, id: isize) -> Option<&mut T> {
        self.inner.get_mut(&id)
    }
}
