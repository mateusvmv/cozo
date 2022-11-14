/* Copyright 2022, The Cozo Project Authors. Licensed under MIT/Apache-2.0/BSD-3-Clause. */

#ifndef COZO_C_H
#define COZO_C_H

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Open a database.
 *
 * `path`:  should contain the UTF-8 encoded path name as a null-terminated C-string.
 * `db_id`: will contain the id of the database opened.
 *
 * When the function is successful, null pointer is returned,
 * otherwise a pointer to a C-string containing the error message will be returned.
 * The returned C-string must be freed with `cozo_free_str`.
 */
char *cozo_open_db(const char *path, int32_t *db_id);

/**
 * Close a database.
 *
 * `id`: the ID representing the database to close.
 *
 * Returns `true` if the database is closed,
 * `false` if it has already been closed, or does not exist.
 */
bool cozo_close_db(int32_t id);

/**
 * Run query against a database.
 *
 * `db_id`: the ID representing the database to run the query.
 * `script_raw`: a UTF-8 encoded C-string for the CozoScript to execute.
 * `params_raw`: a UTF-8 encoded C-string for the params of the query,
 *               in JSON format. You must always pass in a valid JSON map,
 *               even if you do not use params in your query
 *               (pass "{}" in this case).
 * `errored`:    will point to `false` if the query is successful,
 *               `true` if an error occurred.
 *
 * Returns a UTF-8-encoded C-string that **must** be freed with `cozo_free_str`.
 * The string contains the JSON return value of the query.
 */
char *cozo_run_query(int32_t db_id, const char *script_raw, const char *params_raw);

/**
 * Free any C-string returned from the Cozo C API.
 * Must be called exactly once for each returned C-string.
 *
 * `s`: the C-string to free.
 */
void cozo_free_str(char *s);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* COZO_C_H */
