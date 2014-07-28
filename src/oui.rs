use super::*;
use std::mem::{zeroed};
//use std::ptr;

//#define UI_MAX_KIND 16

struct Item {
    // declaration independent unique handle (for persistence)
    handle: Handle,
    // handler
    handler: Handler,

    // container structure

    // number of kids
    numkids: i32,
    // index of first kid
    firstkid: i32,
    // index of last kid
    lastkid: i32,

    // child structure

    // parent item
    parent: i32,
    // index of kid relative to parent
    kidid: i32,
    // index of next sibling with same parent
    nextitem: i32,
    // index of previous sibling with same parent
    previtem: i32,

    // one or multiple of UIlayoutFlags
    layout_flags: u32,
    // size
    size: Vec2,
    // visited flags for layouting
    visited: i32,
    // margin offsets, interpretation depends on flags
    margins: [i32, ..4],
    // neighbors to position borders to
    relto: [i32, ..4],

    // computed size
    computed_size: Vec2,
    // relative rect
    rect: Rect,

    // attributes

    frozen: bool,
    // index of data or -1 for none
    data: i32,
    // size of data
    datasize: i32,
    // a combination of Events
    event_flags: EventFlags,
}
impl Item {
    fn new() -> Item {
        let mut item: Item = unsafe { zeroed() };
        item.parent = -1;
        item.firstkid = -1;
        item.lastkid = -1;
        item.nextitem = -1;
        item.previtem = -1;
        item.data = -1;
        for i in range(0u, 4u) {
            item.relto[i] = -1;
        }
        item
    }
}

enum State {
    STATE_IDLE = 0,
    STATE_CAPTURE,
}

struct Context {
    // button state in this frame
    buttons: u64,
    // button state in the previous frame
    last_buttons: u64,

    // where the cursor was at the beginning of the active state
    start_cursor: Vec2,
    // where the cursor was last frame
    last_cursor: Vec2,
    // where the cursor is currently
    cursor: Vec2,

    hot_handle: Handle,
    active_handle: Handle,
    hot_item: i32,
    active_item: i32,
    hot_rect: Rect,
    active_rect: Rect,
    state: State,

    count: i32,
    items: [Item, ..MAX_ITEMS],
    datasize: i32,
    data: [u8, ..MAX_BUFFERSIZE],
}

//INLINE
pub fn ui_max(a: i32, b: i32) -> i32 { if a>b {a} else {b} }
//INLINE
pub fn ui_min(a: i32, b: i32) -> i32 { if a<b {a} else {b} }


//static ui_context: *mut Context = uiCreateContext();

impl Context {

    pub fn uiCreateContext() -> Context {
        //Context *ctx = (Context *)malloc(sizeof(Context));
        //memset(ctx, 0, sizeof(Context));
        //return ctx;
        Context {
            // button state in this frame
            buttons: 0,
            // button state in the previous frame
            last_buttons: 0,

            // where the cursor was at the beginning of the active state
            start_cursor: Vec2::zero(),
            // where the cursor was last frame
            last_cursor: Vec2::zero(),
            // where the cursor is currently
            cursor: Vec2::zero(),

            hot_handle: -1,
            active_handle: -1,
            hot_item: -1,
            active_item: -1,
            hot_rect: Rect::zero(),
            active_rect: Rect::zero(),
            state: STATE_IDLE,

            count: 0,
            items: [Item::new(), ..MAX_ITEMS as uint],
            datasize: 0,
            data: [0, ..MAX_BUFFERSIZE as uint],
        }
    }
    //static NULL: *const Any = ptr::null();

    //pub fn uiMakeCurrent(ctx: *const Context) {
    //    ui_context = ctx;
    //    if ui_context != ptr::null() {
    //        uiClear();
    //    }
    //}

    //pub fn uiDestroyContext(ctx: *const Context) {
    //    if ui_context == ctx {
    //        uiMakeCurrent(ptr::null());
    //    }
    //    //free(ctx);
    //}

    pub fn uiSetButton(&mut self, button: u64, enabled: bool) {
        let mask = 1u64<<button as uint;
        // set new bit
        self.buttons = if enabled
                {self.buttons | mask}
            else {self.buttons & !mask};
    }

    pub fn uiGetLastButton(&self, button: u64) -> bool {
        self.last_buttons & (1u64<<button as uint) != 0
    }

    pub fn uiGetButton(&self, button: u64) -> bool {
        self.buttons & (1u64<<button as uint) != 0
    }

    pub fn uiButtonPressed(&self, button: u64) -> bool {
        !self.uiGetLastButton(button) && self.uiGetButton(button)
    }

    pub fn uiButtonReleased(&self, button: u64) -> bool {
        self.uiGetLastButton(button) && !self.uiGetButton(button)
    }

    pub fn uiSetCursor(&mut self, x: i32, y: i32) {
        self.cursor.x = x;
        self.cursor.y = y;
    }

    pub fn uiGetCursor(&self) -> Vec2 {
        self.cursor
    }

    pub fn uiGetCursorStart(&self) -> Vec2 {
        self.start_cursor
    }

    pub fn uiGetCursorDelta(&self) -> Vec2 {
        Vec2 {
            x: self.cursor.x - self.last_cursor.x,
            y: self.cursor.y - self.last_cursor.y
        }
    }

    pub fn uiGetCursorStartDelta(&self) -> Vec2 {
        Vec2 {
            x: self.cursor.x - self.start_cursor.x,
            y: self.cursor.y - self.start_cursor.y
        }
    }

    pub fn uiItemRef(&mut self, item: i32) -> &mut Item {
        assert!((item >= 0) && (item < self.count));
        let item = item as uint;
        return &mut self.items[item];
    }

    pub fn uiClear(&mut self) {
        self.count = 0;
        self.datasize = 0;
        self.hot_item = -1;
        self.active_item = -1;
    }

    pub fn uiItem(&mut self) -> i32 {
        assert!((self.count as u32) < MAX_ITEMS);
        let idx = self.count;
                  self.count += 1;
        let item = self.uiItemRef(idx);
        *item = unsafe { zeroed() };
        item.parent = -1;
        item.firstkid = -1;
        item.lastkid = -1;
        item.nextitem = -1;
        item.previtem = -1;
        item.data = -1;
        for i in range(0u, 4u) {
            item.relto[i] = -1;
        }
        return idx;
    }

    pub fn uiNotifyItem(&mut self, item: i32, event: EventFlags) {
        let pitem = self.uiItemRef(item);
        if pitem.handler.is_some() && pitem.event_flags.contains(event) {
            (pitem.handler.unwrap())(item, event);
        }
    }

    pub fn uiAppend(&mut self, item: i32, child: i32) -> i32 {
        assert!(child > 0);
        assert!(self.uiParent(child) == -1);
        {
//            let pchild = self.uiItemRef(child);
//            let pparent = self.uiItemRef(item);
//            pchild.parent = item;
//            pchild.kidid = pparent.numkids;
//                           pparent.numkids+= 1;
//            if (pparent.lastkid < 0) {
//                pparent.firstkid = child;
//                pparent.lastkid = child;
//            } else {
//                pchild.previtem = pparent.lastkid;
//                self.uiItemRef(pparent.lastkid).nextitem = child;
//                pparent.lastkid = child;
//            }
        }
        self.uiNotifyItem(item, APPEND);
        return child;
    }

    pub fn uiSetFrozen(&mut self, item: i32, enable: bool) {
        let pitem = self.uiItemRef(item);
        pitem.frozen = enable;
    }

    pub fn uiSetSize(&mut self, item: i32, w: i32, h: i32) {
        let pitem = self.uiItemRef(item);
        pitem.size.x = w;
        pitem.size.y = h;
    }

    pub fn uiGetWidth(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).size.x;
    }

    pub fn uiGetHeight(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).size.y;
    }

    pub fn uiSetLayout(&mut self, item: i32, flags: u32) {
        self.uiItemRef(item).layout_flags = flags;
    }

    pub fn uiGetLayout(&mut self, item: i32) -> u32 {
        return self.uiItemRef(item).layout_flags;
    }

    pub fn uiSetMargins(&mut self, item: i32, l: i32, t: i32, r: i32, b: i32) {
        let pitem = self.uiItemRef(item);
        pitem.margins[0] = l;
        pitem.margins[1] = t;
        pitem.margins[2] = r;
        pitem.margins[3] = b;
    }

    pub fn uiGetMarginLeft(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).margins[0];
    }
    pub fn uiGetMarginTop(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).margins[1];
    }
    pub fn uiGetMarginRight(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).margins[2];
    }
    pub fn uiGetMarginDown(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).margins[3];
    }


    pub fn uiSetRelToLeft(&mut self, item: i32, other: i32) {
        assert!((other < 0) || (self.uiParent(other) == self.uiParent(item)));
        self.uiItemRef(item).relto[0] = other;
    }

    pub fn uiGetRelToLeft(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).relto[0];
    }

    pub fn uiSetRelToTop(&mut self, item: i32, other: i32) {
        assert!((other < 0) || (self.uiParent(other) == self.uiParent(item)));
        self.uiItemRef(item).relto[1] = other;
    }
    pub fn uiGetRelToTop(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).relto[1];
    }

    pub fn uiSetRelToRight(&mut self, item: i32, other: i32) {
        assert!((other < 0) || (self.uiParent(other) == self.uiParent(item)));
        self.uiItemRef(item).relto[2] = other;
    }
    pub fn uiGetRelToRight(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).relto[2];
    }

    pub fn uiSetRelToDown(&mut self, item: i32, other: i32) {
        assert!((other < 0) || (self.uiParent(other) == self.uiParent(item)));
        self.uiItemRef(item).relto[3] = other;
    }
    pub fn uiGetRelToDown(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).relto[3];
    }


    //INLINE
    pub fn uiComputeChainSize<'a>(&'a mut self, pkid: &'a mut Item,
        need_size: &mut i32, hard_size: &mut i32, dim: uint
    ) {
//        let mut pitem = pkid;
//        let wdim = dim+2;
//        let mut size = pitem.rect[wdim] + pitem.margins[dim] + pitem.margins[wdim];
//        *need_size = size;
//        *hard_size = if pitem.size[dim] > 0 {size} else {0};
//
//        let mut it = 0u32;
//        pitem.visited |= 1<<dim;
//        // traverse along left neighbors
//        while ((pitem.layout_flags>>dim) & LEFT.bits) != 0 {
//            if (pitem.relto[dim] < 0) {break};
//            pitem = self.uiItemRef(pitem.relto[dim]);
//            pitem.visited |= 1<<dim;
//            size = pitem.rect[wdim] + pitem.margins[dim] + pitem.margins[wdim];
//            *need_size = (*need_size) + size;
//            *hard_size = (*hard_size) + (if pitem.size[dim] > 0 {size} else {0});
//            it += 1;
//            assert!(it<1000000); // infinite loop
//        }
//        // traverse along right neighbors
//        pitem = pkid;
//        it = 0;
//        while LayoutFlags::from_bits(pitem.layout_flags>>dim).expect("bitfail").contains(RIGHT) {
//            if (pitem.relto[wdim] < 0) {break};
//            pitem = self.uiItemRef(pitem.relto[wdim]);
//            pitem.visited |= 1<<dim;
//            size = pitem.rect[wdim] + pitem.margins[dim] + pitem.margins[wdim];
//            *need_size = (*need_size) + size;
//            *hard_size = (*hard_size) + (if pitem.size[dim] > 0 {size} else {0});
//            it += 1;
//            assert!(it<1000000); // infinite loop
//        }
    }

    //INLINE
    pub fn uiComputeSizeDim(&mut self, pitem: &mut Item, dim: uint) {
//        let wdim = dim+2;
//        let mut need_size = 0;
//        let mut hard_size = 0;
//        let kid = pitem.firstkid;
//        while (kid >= 0) {
//            let pkid = self.uiItemRef(kid);
//            if pkid.visited & (1<<dim) == 0 {
//                let mut ns: i32 = 0;
//                let mut hs: i32 = 0;
//                self.uiComputeChainSize(pkid, &mut ns, &mut hs, dim);
//                need_size = ui_max(need_size, ns);
//                hard_size = ui_max(hard_size, hs);
//            }
//            kid = self.uiNextSibling(kid);
//        }
//        pitem.computed_size[dim] = hard_size;
//
//        if (pitem.size[dim] > 0) {
//            pitem.rect[wdim] = pitem.size[dim];
//        } else {
//            pitem.rect[wdim] = need_size;
//        }
    }

    //static
    pub fn uiComputeBestSize(&mut self, item: i32, dim: uint) {
//        let pitem = self.uiItemRef(item);
//        pitem.visited = 0;
//        // children expand the size
//        let mut kid = self.uiFirstChild(item);
//        while (kid >= 0) {
//            self.uiComputeBestSize(kid, dim);
//            kid = self.uiNextSibling(kid);
//        }
//
//        self.uiComputeSizeDim(pitem, dim);
    }

    //static
    pub fn uiLayoutChildItem(&mut self, pparent: &Item, pitem: &mut Item, dyncount: &mut i32, dim: uint) {
//        if (pitem.visited & (4<<dim) != 0) {return};
//        pitem.visited |= (4<<dim);
//
//        if (pitem.size[dim] == 0) {
//            *dyncount = (*dyncount)+1;
//        }
//
//        let wdim = dim+2;
//
//        let x = 0;
//        let s = pparent.rect[wdim];
//
//        let flags = LayoutFlags::from_bits((pitem.layout_flags>>dim) as u32).expect("fail");
//        let hasl = flags.contains(LEFT) && (pitem.relto[dim] >= 0);
//        let hasr = flags.contains(RIGHT) && (pitem.relto[wdim] >= 0);
//
//        if (hasl) {
//            let pl = self.uiItemRef(pitem.relto[dim]);
//            self.uiLayoutChildItem(pparent, pl, dyncount, dim);
//            x = pl.rect[dim]+pl.rect[wdim]+pl.margins[wdim];
//            s -= x;
//        }
//        if (hasr) {
//            let pl = self.uiItemRef(pitem.relto[wdim]);
//            self.uiLayoutChildItem(pparent, pl, dyncount, dim);
//            s = pl.rect[dim]-pl.margins[dim]-x;
//        }
//
//        match flags & HFILL {
//            LEFT => {
//                pitem.rect[dim] = x+pitem.margins[dim];
//            }
//            RIGHT => {
//                pitem.rect[dim] = x+s-pitem.rect[wdim]-pitem.margins[wdim];
//            }
//            HFILL => {
//                if (pitem.size[dim] > 0) { // hard maximum size; can't stretch
//                    if (!hasl) {
//                        pitem.rect[dim] = x+pitem.margins[dim];
//                    }
//                    else {
//                        pitem.rect[dim] = x+s-pitem.rect[wdim]-pitem.margins[wdim];
//                    }
//                } else {
//                    if (true) { // !pitem.rect[wdim]) {
//                        let width = (pparent.rect[wdim] - pparent.computed_size[dim]);
//                        let space = width / (*dyncount);
//                        //let rest = width - space*(*dyncount);
//                        if (!hasl) {
//                            pitem.rect[dim] = x+pitem.margins[dim];
//                            pitem.rect[wdim] = s-pitem.margins[dim]-pitem.margins[wdim];
//                        } else {
//                            pitem.rect[wdim] = space-pitem.margins[dim]-pitem.margins[wdim];
//                            pitem.rect[dim] = x+s-pitem.rect[wdim]-pitem.margins[wdim];
//                        }
//                    } else {
//                        pitem.rect[dim] = x+pitem.margins[dim];
//                        pitem.rect[wdim] = s-pitem.margins[dim]-pitem.margins[wdim];
//                    }
//                }
//            }
//            //default:
//            _ /*HCENTER*/ => {
//                pitem.rect[dim] = x+(s-pitem.rect[wdim])/2+pitem.margins[dim];
//            }
//        }
    }

    //INLINE
    pub fn uiLayoutItemDim(&mut self, pitem: &mut Item, dim: uint) {
//        let mut kid = pitem.firstkid;
//        while (kid >= 0) {
//            let pkid = self.uiItemRef(kid);
//            let mut dyncount = 0;
//            self.uiLayoutChildItem(pitem, pkid, &dyncount, dim);
//            kid = self.uiNextSibling(kid);
//        }
    }

    //static
    pub fn uiLayoutItem(&mut self, item: i32, dim: uint) {
//        let pitem = self.uiItemRef(item);
//
//        self.uiLayoutItemDim(pitem, dim);
//
//        let mut kid = self.uiFirstChild(item);
//        while (kid >= 0) {
//            self.uiLayoutItem(kid, dim);
//            kid = self.uiNextSibling(kid);
//        }
    }

    pub fn uiGetRect(&mut self, item: i32) -> Rect {
        return self.uiItemRef(item).rect;
    }

    pub fn uiGetActiveRect(&self) -> Rect {
        return self.active_rect;
    }

    pub fn uiFirstChild(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).firstkid;
    }

    pub fn uiLastChild(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).lastkid;
    }

    pub fn uiNextSibling(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).nextitem;
    }

    pub fn uiPrevSibling(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).previtem;
    }

    pub fn uiParent(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).parent;
    }

    pub fn uiGetData(&mut self, item: i32) -> &mut [u8] {
        let (data, datasize) = {
            let pitem = self.uiItemRef(item);
            (pitem.data, pitem.datasize)
        };
//        if (pitem.data < 0) {return NODATA;}
        return self.data.mut_slice(data as uint, datasize as uint);
    }

    pub fn uiAllocData(&mut self, item: i32, size: i32) -> &[u8] {
        assert!((size > 0) && ((size as uint) < (MAX_DATASIZE as uint)));
        let alloc = self.datasize;
        self.datasize += size;
        {
            let pitem = self.uiItemRef(item);
            assert!(pitem.data < 0);
            assert!((alloc+size) as uint <= MAX_BUFFERSIZE as uint);
            pitem.data = alloc;
        }
        return self.data.slice(alloc as uint, size as uint);
    }

    pub fn uiSetHandle(&mut self, item: i32, handle: Handle) {
        self.uiItemRef(item).handle = handle;
        if handle != -1 {
            if handle == self.hot_handle {
                self.hot_item = item;
            }
            if handle == self.active_handle {
                self.active_item = item;
            }
        }
    }

    pub fn uiGetHandle(&mut self, item: i32) -> Handle {
        return self.uiItemRef(item).handle;
    }

    pub fn uiSetHandler(&mut self, item: i32, handler: Handler, flags: EventFlags) {
        let pitem =self. uiItemRef(item);
        pitem.handler = handler;
        pitem.event_flags = flags;
    }

    pub fn uiGetHandler(&mut self, item: i32) -> Handler {
        return self.uiItemRef(item).handler;
    }

    pub fn uiGetHandlerFlags(&mut self, item: i32) -> EventFlags {
        return self.uiItemRef(item).event_flags;
    }

    pub fn uiGetChildId(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).kidid;
    }

    pub fn uiGetChildCount(&mut self, item: i32) -> i32 {
        return self.uiItemRef(item).numkids;
    }

    pub fn uiFindItem(&mut self, item: i32, x: i32, y: i32, ox: i32, oy: i32) -> i32 {
//        let mut pitem = self.uiItemRef(item);
//        if (pitem.frozen) {return -1;}
//        let mut rect = pitem.rect;
//        let x = x - rect.x;
//        let y = y - rect.y;
//        let ox = ox + rect.x;
//        let oy = oy + rect.y;
//        if ((x>=0)
//         && (y>=0)
//         && (x<rect.w)
//         && (y<rect.h)) {
//            let kid = self.uiFirstChild(item);
//            while (kid >= 0) {
//                let best_hit = self.uiFindItem(kid,x,y,ox,oy);
//                if (best_hit >= 0) {return best_hit;}
//                kid = self.uiNextSibling(kid);
//            }
//            rect.x += ox;
//            rect.y += oy;
//            self.hot_rect = rect;
//            return item;
//        }
        return -1;
    }

    pub fn uiLayout(&mut self) {
        if self.count == 0 { return; }

        // compute widths
        self.uiComputeBestSize(0,0);
        // position root element rect
        self.uiItemRef(0).rect.x = self.uiItemRef(0).margins[0];
        self.uiLayoutItem(0,0);

        // compute heights
        self.uiComputeBestSize(0,1);
        // position root element rect
        self.uiItemRef(0).rect.y = self.uiItemRef(0).margins[1];
        self.uiLayoutItem(0,1);
    }

    pub fn uiProcess(&mut self) {
        if self.count == 0 { return; }

        let cursor = self.cursor;
        let hot = self.uiFindItem(0, cursor.x, cursor.y, 0, 0);
        let active = self.active_item;

        match self.state {
            //default:
            STATE_IDLE => {
                self.start_cursor = cursor;
                if self.uiGetButton(0) {
                    self.hot_item = -1;
                    self.active_rect = self.hot_rect;
                    self.active_item = hot;
                    if hot >= 0 {
                        self.uiNotifyItem(hot, BUTTON0_DOWN);
                    }
                    self.state = STATE_CAPTURE;
                } else {
                    self.hot_item = hot;
                }
            }
            STATE_CAPTURE => {
                if !self.uiGetButton(0) {
                    if active >= 0 {
                        self.uiNotifyItem(active, BUTTON0_UP);
                        if active == hot {
                            self.uiNotifyItem(active, BUTTON0_HOT_UP);
                        }
                    }
                    self.active_item = -1;
                    self.state = STATE_IDLE;
                } else {
                    if active >= 0 {
                        self.uiNotifyItem(active, BUTTON0_CAPTURE);
                    }
                    if hot == active {
                        self.hot_item = hot;
                    }
                    else {
                        self.hot_item = -1;
                    }
                }
            }
        }
        // self has changed, reset handles to match current state
        self.last_cursor = self.cursor;
        let active = self.active_item;
        let hot = self.hot_item;
        self.hot_handle = if hot>=0 {self.uiGetHandle(hot)} else {0};
        self.active_handle = if active>=0 {self.uiGetHandle(active)} else {0};
    }

    //static
    pub fn uiIsActive(&self, item: i32) -> bool {
        return self.active_item == item;
    }

    //static
    pub fn uiIsHot(&self, item: i32) -> bool {
        return self.hot_item == item;
    }

    pub fn uiGetState(&mut self, item: i32) -> ItemState {
        let hot = self.uiIsHot(item);
        let active = self.uiIsActive(item);
        let pitem = self.uiItemRef(item);
        if pitem.frozen {return FROZEN;}
        if active {
            if pitem.event_flags.contains(BUTTON0_CAPTURE|BUTTON0_UP) {return ACTIVE;}
            if pitem.event_flags.contains(BUTTON0_HOT_UP) && hot {
                return ACTIVE;
            }
            return COLD;
        } else if hot {
            return HOT;
        }
        return COLD;
    }
}