use core::cmp::Ordering;
use heapless::binary_heap::{BinaryHeap, Max};
use heapless::FnvIndexMap;
use heapless::consts::*;
use graphics_server::*;
use log::{error, info};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CanvasState {
    // the initial state of every Canvas. Not drawable.
    Created,
    // this state indicates the Canvas can be drawn, and may or may not need to be flushed to the screen.
    DrawableDirty,
    // this state indicates the Canvas has been flushed to the screen.
    DrawableDrawn,
    // indicates that the Canvas is not drawable, but needs to be defaced
    NotDrawableDirty,
    // indicates that the Canvas is not drawable, and has been defaced
    NotDrawableDefaced,
}

/// A rectangular region that defines a top-left zero relative offset for graphical items
/// and a bottom-right point that defines a clipping area for things drawn inside.
#[derive(Debug, Copy, Clone)]
pub struct Canvas {
    // unique, random identifier for the Canvas
    gid: Gid,

    // screen coordinates of the clipping region
    clip_rect: Rectangle,

    // trust level, 255 is most trusted
    trust_level: u8,

    // enables scroll/pan of objects within a region
    pan_offset: Point,

    // track the drawing state of the canvas
    state: CanvasState,
}

#[allow(dead_code)]
impl Canvas {
    pub fn new(clip_rect: Rectangle, trust_level: u8,
        trng_conn: xous::CID, pan_offset: Option<Point>) -> Result<Canvas, xous::Error> {

        let mut gid: [u32; 4] = [0; 4];
        let g: u64 = trng::get_u64(trng_conn)?;
        gid[0] = g as u32;
        gid[1] = (g >> 32) as u32;
        let g: u64 = trng::get_u64(trng_conn)?;
        gid[2] = g as u32;
        gid[3] = (g >> 32) as u32;

        Ok(if pan_offset.is_some() {
            Canvas {
                clip_rect, trust_level, state: CanvasState::Created, gid: Gid::new(gid), pan_offset: pan_offset.unwrap()
            }
        } else {
            Canvas {
                clip_rect, trust_level, state: CanvasState::Created, gid: Gid::new(gid), pan_offset: Point::new(0, 0)
            }
        })
    }
    pub fn pan_offset(&self) -> Point { self.pan_offset }
    pub fn clip_rect(&self) -> Rectangle { self.clip_rect }
    pub fn set_clip(&mut self, cr: Rectangle) { self.clip_rect = cr; self.state = CanvasState::Created }
    pub fn gid(&self) -> Gid { self.gid }
    pub fn trust_level(&self) -> u8 { self.trust_level }
    pub fn state(&self) -> CanvasState { self.state }
    pub fn is_drawable(&self) -> bool {
        if self.state == CanvasState::DrawableDirty || self.state == CanvasState::DrawableDrawn {
            true
        } else {
            false
        }
    }
    pub fn set_drawable(&mut self, drawable: bool) {
        if drawable {
            if self.state != CanvasState::DrawableDrawn {
                self.state = CanvasState::DrawableDirty;
            }
        } else {
            if self.state != CanvasState::NotDrawableDefaced {
                self.state = CanvasState::NotDrawableDirty;
            }
        }
    }
    // call this after the screen has been flushed
    pub fn do_flushed(&mut self) -> Result<(), xous::Error> {
        if self.state == CanvasState::DrawableDirty || self.state == CanvasState::DrawableDrawn {
            self.state = CanvasState::DrawableDrawn;
            Ok(())
        } else if self.state == CanvasState::NotDrawableDefaced {
            Ok(())
        } else {
            error!("Canvas: flush happened before not drawable regions were defaced, or before initialized!");
            Err(xous::Error::UseBeforeInit)
        }
    }
    pub fn do_drawn(&mut self) -> Result<(), xous::Error> {
        if self.state == CanvasState::DrawableDirty || self.state == CanvasState::DrawableDrawn {
            self.state = CanvasState::DrawableDirty;
            Ok(())
        } else {
            error!("Canvas: attempt to draw on regions that are not drawable, or not initialized!");
            Err(xous::Error::AccessDenied)
        }
    }
    pub fn do_defaced(&mut self) -> Result<(), xous::Error> {
        if self.state == CanvasState::NotDrawableDirty {
            self.state = CanvasState::NotDrawableDefaced;
            Ok(())
        } else if self.state == CanvasState::DrawableDirty || self.state == CanvasState::DrawableDrawn {
            info!("Canvas: drawable region was defaced. Allowing it, but this could be a logic bug");
            Ok(())
        } else {
            error!("Canvas: attempt to deface region already defaced, or not initialized!");
            Err(xous::Error::DoubleFree)
        }
    }
    pub fn needs_defacing(&self) -> bool {
        if self.state == CanvasState::NotDrawableDirty {
            true
        } else {
            false
        }
    }
}

impl Ord for Canvas {
    fn cmp(&self, other: &Self) -> Ordering {
        self.trust_level.cmp(&other.trust_level)
    }
}
impl PartialOrd for Canvas {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Canvas {
    fn eq(&self, other: &Self) -> bool {
        self.trust_level == other.trust_level
    }
}
impl Eq for Canvas {}


pub fn deface(gfx_conn: xous::CID, canvases: &mut FnvIndexMap<Gid, Canvas, U32>) -> Result<(), xous::Error> {
    // first check if any need defacing, if not, then we're done
    let mut needs_defacing = false;
    for (_, c) in canvases.iter() {
        if c.needs_defacing() {
            needs_defacing = true;
        }
    }
    if needs_defacing {
        error!("GAM: haven't implemented canvas defacing yet");
        /*
        This routine will need to do something similar to recompute_canvases, where it extracts
        a sorted order and draws the defacement upon the canvas that requires defacing.

        For simplicity, we may be able to assume this is called with at most one layout change
        in between states, so in the worst case we are drawing defacement with a rectangular clip
        area open in the middle of a canvas...
         */
        //Err(xous::Error::InternalError)

        // temporarirly clear the error so we can develop other features
        for (_, c) in canvases.iter_mut() {
            if c.needs_defacing() {
                c.do_defaced();
            }
        }
        Ok(())
    } else {
        Ok(())
    }
}

// we use the "screen" parameter to determine when we can turn off drawing to canvases that are off-screen
pub fn recompute_canvases(canvases: FnvIndexMap<Gid, Canvas, U32>, screen: Rectangle) -> FnvIndexMap<Gid, Canvas, U32> {
    let debug = false;
    // first, sort canvases by trust_level. Canvas implements ord/eq based on the trust_level attribute
    // so jush pushing it into a max binary heap does the trick.
    if debug { info!("CANVAS: recompute canvas"); }
    let mut sorted_clipregions: BinaryHeap<Canvas, U32, Max> = BinaryHeap::new();
    for (&k, &c) in canvases.iter() {
        if debug { info!("   CANVAS: sorting gid {:?}, canvas {:?}", k, c);}
        sorted_clipregions.push(c).unwrap(); // always succeeds because incoming type is the same size
    }

    // now, descend through trust levels and compute intersections, putting the updated drawable states into higher_clipregions
    let mut higher_clipregions: BinaryHeap<Canvas, U32, Max> = BinaryHeap::new();
    let mut trust_level: u8 = 255;
    // sorted_clipregions is a Max heap keyed on trust, so popping the elements off will return them sorted from most to least trusted
    if debug{info!("CANVAS: received screen argument of {:?}", screen);}
    if debug{info!("CANVAS: now determining which regions are drawable");}
    loop {
        if let Some(c) = sorted_clipregions.pop() {
            if debug { info!("   CANVAS: considering {:?}", c);}
            let mut canvas = c.clone();

            let mut drawable: bool = true;
            let clip_region = canvas.clip_rect();
            if trust_level < canvas.trust_level() {
                trust_level = canvas.trust_level();
            }
            if !clip_region.intersects(screen) {
                drawable = false;
                if debug { info!("    * CANVAS: not drawable, does not intersect");}
            } else { // short circuit this computation if it's not drawable because it's off screen
                // note that this .iter() is *not* sorted by trust level, but all elements will be of greater than or equal to the current trust level
                for &region in higher_clipregions.iter() {
                    // regions of the same trust level can draw over each other. Draw order is arbitrary.
                    if region.clip_rect().intersects(clip_region) && (region.trust_level() < trust_level) {
                        drawable = false;
                        if debug { info!("    * CANVAS: not drawable, lower trust intersecting with higher trust region");}
                    }
                }
            }
            canvas.set_drawable(drawable);
            higher_clipregions.push(canvas).unwrap();
        } else {
            break;
        }
    }

    // create a new index map out of the recomputed higher_clipregions
    let mut map: FnvIndexMap<Gid, Canvas, U32> = FnvIndexMap::new();
    if debug { info!("CANVAS: reconstituting index map");}
    for &c in higher_clipregions.iter() {
        if debug { info!("   CANVAS: inserting gid {:?}, canvas {:?}", c.gid(), c);}
        map.insert(c.gid(), c).unwrap();
    }

    map
}