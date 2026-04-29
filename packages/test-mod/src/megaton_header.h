#pragma once

// this will probably be put into a header in megaton
namespace megaton {

/**
 * Call the Rust entry point to initialize the Rust code.
 *
 * This should be called as part of your `megaton_main` C entry point.
 */
void rust_main();

}
