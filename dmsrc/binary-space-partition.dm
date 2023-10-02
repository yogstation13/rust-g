/**
 * This proc generates rooms in a specified area of random size and placement. Essential for procedurally generated areas, BSP works by cutting a given area in half,
 * then cutting one of those subsections in half, and repeating this process until a minimum size is reached, then backtracking to other subsections that are not of
 * the minimum size yet. These cuts are offset by small random amounts so that the sections are all varied in size and shape.
 *
 * BSP excels at creating rooms or areas with a relatively even distribution over an area, so there won't be too much blank open area. However if you discard rooms that
 * overlap pre-existing map structures or areas, you may still get blank areas where nothing interesting appears.
 *
 * Return:
 * * a json list of room data to be processed by json_decode in byond and further processed there.
 *
 * Arguments:
 * * width: the width of the area to generate in
 * * height: the height of the area to generate in
 * * hash: the rng seed the generator will use for this instance
 * * map_subsection_min_size: The minimum size of a map subsection. When using this for rooms with walls, the minimum possible square will be a 5x5 room. Barring walls,
 * this will be a 3x3 room. The maximum size will be 9x9, because a further cut could reduce this size beneath the minimum size.
 * * map_subsection_min_room_width: The minimum room width once the subsections are finalized. Room width and height are random between this amount, and the subsection
 * max size
 * * map_subsection_min_room_height: The minimum room height once the subsections are finalized. Room width and height are random between this amount, and the subsection
 * max size
 */
#define rustg_bsp_generate(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height) \
	RUSTG_CALL(RUST_G, "bsp_generate")(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height)
