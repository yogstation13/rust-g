/**
 * This proc generates rooms in a specified area of random size and placement. Used in procedural generation, but far less intensively than Binary Space Partitioning
 * due to Random Room Placement being far more simple and unreliable for area coverage. These rooms will not overlap one another, but that is the only logic
 * they do. The room dimensions returned by this call are hardcoded to be the dimensions of maint ruins so that I could sprinkle pre-generated areas over
 * the binary space rooms that are random.
 * These dimensions are:
 * * 3x3
 * * 3x5
 * * 5x3
 * * 5x4
 * * 10x5
 * * 10x10
 *
 * Return:
 * * a json list of room data to be processed by json_decode in byond and further processed there.
 *
 * Arguments:
 * * width: the width of the area to generate in
 * * height: the height of the area to generate in
 * * desired_room_count: the number of rooms you want generated and returned
 * * hash: the rng seed the generator will use for this instance
 */
#define rustg_random_room_generate(width, height, desired_room_count, hash) \
	RUSTG_CALL(RUST_G, "random_room_generate")(width, height, desired_room_count, hash)
