#include "bridge.h"

#include <limits.h>
#include <stdlib.h>
#include <unicode/ucol.h>
#include <unicode/usearch.h>
#include <unicode/utf16.h>

typedef struct {
  UChar *text;
  UChar *pattern;
  int32_t text_length;
  int32_t pattern_length;
  UCollator *collator;
  UStringSearch *search;
} SearchState;

static void close_search(SearchState *state) {
  if (state->search != NULL)
    usearch_close(state->search);
  if (state->collator != NULL)
    ucol_close(state->collator);
  free(state->pattern);
  free(state->text);
}

static UBool open_collator(SearchState *state, const char *locale,
                            int32_t locale_length, int32_t strength,
                            WowIcuError *error) {
  static const UCollationStrength strengths[] = {
      UCOL_PRIMARY, UCOL_SECONDARY, UCOL_TERTIARY, UCOL_QUATERNARY, UCOL_IDENTICAL};
  if (strength < 0 || strength > 4) {
    wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR, "select search strength");
    return 0;
  }
  char *parsed = wow_icu_locale(locale, locale_length, error);
  if (parsed == NULL)
    return 0;
  UErrorCode status = U_ZERO_ERROR;
  state->collator = ucol_open(parsed, &status);
  free(parsed);
  if (U_FAILURE(status) || state->collator == NULL) {
    wow_icu_fail(error, U_FAILURE(status) ? status : U_INTERNAL_PROGRAM_ERROR,
                 "open search collator");
    return 0;
  }
  ucol_setStrength(state->collator, strengths[strength]);
  ucol_setAttribute(state->collator, UCOL_NORMALIZATION_MODE, UCOL_ON, &status);
  if (U_FAILURE(status)) {
    wow_icu_fail(error, status, "enable search normalization");
    return 0;
  }
  return 1;
}

static UBool open_search(SearchState *state, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  state->search = usearch_openFromCollator(
      state->pattern, state->pattern_length, state->text, state->text_length,
      state->collator, NULL, &status);
  if (U_FAILURE(status) || state->search == NULL) {
    wow_icu_fail(error, U_FAILURE(status) ? status : U_INTERNAL_PROGRAM_ERROR,
                 "open collation search");
    return 0;
  }
  usearch_setAttribute(state->search, USEARCH_OVERLAP, USEARCH_OFF, &status);
  if (U_FAILURE(status)) {
    wow_icu_fail(error, status, "disable overlapping search matches");
    return 0;
  }
  return 1;
}

static UBool scalar_boundary(const SearchState *state, int32_t offset) {
  if (offset < 0 || offset > state->text_length)
    return 0;
  if (offset == 0 || offset == state->text_length)
    return 1;
  return !(U16_IS_LEAD(state->text[offset - 1]) &&
           U16_IS_TRAIL(state->text[offset]));
}

static UBool append_match(WowIcuMatches *output, int32_t *capacity,
                           int32_t start, int32_t end, WowIcuError *error) {
  if (output->length == *capacity) {
    if (*capacity > (INT32_MAX - 1) / 2) {
      wow_icu_fail(error, U_INDEX_OUTOFBOUNDS_ERROR, "grow search matches");
      return 0;
    }
    int32_t next = *capacity == 0 ? 16 : *capacity * 2;
    if ((size_t)next > SIZE_MAX / sizeof(WowIcuMatch)) {
      wow_icu_fail(error, U_INDEX_OUTOFBOUNDS_ERROR, "size search matches");
      return 0;
    }
    void *data = realloc(output->data, (size_t)next * sizeof(WowIcuMatch));
    if (data == NULL) {
      wow_icu_fail(error, U_MEMORY_ALLOCATION_ERROR, "allocate search matches");
      return 0;
    }
    output->data = data;
    *capacity = next;
  }
  output->data[output->length++] = (WowIcuMatch){start, end};
  return 1;
}

static UBool collect_matches(SearchState *state, WowIcuMatches *output,
                              WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t capacity = 0;
  int32_t previous = -1;
  int32_t previous_end = 0;
  int32_t start = usearch_first(state->search, &status);
  while (U_SUCCESS(status) && start != USEARCH_DONE) {
    int32_t length = usearch_getMatchedLength(state->search);
    if (start <= previous || start < previous_end || start > state->text_length ||
        length < 0 || length > state->text_length - start) {
      wow_icu_fail(error, U_INTERNAL_PROGRAM_ERROR, "validate search progress");
      return 0;
    }
    int32_t end = start + length;
    if (!scalar_boundary(state, start) || !scalar_boundary(state, end)) {
      wow_icu_fail(error, U_INTERNAL_PROGRAM_ERROR, "validate search scalar boundaries");
      return 0;
    }
    previous = start;
    previous_end = end;
    if (length > 0 && !append_match(output, &capacity, start, end, error))
      return 0;
    start = usearch_next(state->search, &status);
  }
  if (U_FAILURE(status)) {
    wow_icu_fail(error, status, "iterate collation search");
    return 0;
  }
  return 1;
}

int32_t wow_icu_find_matches(const char *locale, int32_t locale_length,
                              const uint8_t *text, int32_t text_length,
                              const uint8_t *pattern, int32_t pattern_length,
                              int32_t strength, WowIcuMatches *output,
                              WowIcuError *error) {
  SearchState state = {0};
  int32_t result = WOW_ICU_ERROR;
  if (!open_collator(&state, locale, locale_length, strength, error))
    goto cleanup;
  state.text = wow_icu_to_utf16(text, text_length, &state.text_length, error);
  if (state.text == NULL)
    goto cleanup;
  state.pattern = wow_icu_to_utf16(pattern, pattern_length, &state.pattern_length, error);
  if (state.pattern == NULL)
    goto cleanup;
  if (state.text_length == 0 || state.pattern_length == 0) {
    result = WOW_ICU_OK;
    goto cleanup;
  }
  if (open_search(&state, error) && collect_matches(&state, output, error))
    result = WOW_ICU_OK;
cleanup:
  close_search(&state);
  if (result != WOW_ICU_OK) {
    free(output->data);
    output->data = NULL;
    output->length = 0;
  }
  return result;
}

void wow_icu_matches_free(WowIcuMatch *matches) { free(matches); }
