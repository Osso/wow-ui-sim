#include "bridge.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unicode/ulistformatter.h>
#include <unicode/unumberformatter.h>

static UListFormatterWidth list_width(int32_t width) {
  if (width == 1)
    return ULISTFMT_WIDTH_SHORT;
  if (width == 2)
    return ULISTFMT_WIDTH_NARROW;
  return ULISTFMT_WIDTH_WIDE;
}

static UNumberFormatter *open_unit_formatter(const char *locale,
                                             const WowIcuDurationPart *part,
                                             int32_t width,
                                             WowIcuError *error) {
  static const char *units[] = {"second", "minute", "hour", "day"};
  const char *unit_width = width == 1 ? "short" : width == 2 ? "narrow" : "full-name";
  const char *precision = part->fraction_digits == 3 ? ".###" : "precision-integer";
  char skeleton[128];
  int length = snprintf(skeleton, sizeof(skeleton),
                        "measure-unit/duration-%s unit-width-%s group-off %s",
                        units[part->unit], unit_width, precision);
  if (length < 0 || (size_t)length >= sizeof(skeleton)) {
    wow_icu_fail(error, U_BUFFER_OVERFLOW_ERROR, "build duration unit skeleton");
    return NULL;
  }
  int32_t utf16_length = 0;
  UChar *utf16 = wow_icu_to_utf16((const uint8_t *)skeleton, length,
                                 &utf16_length, error);
  if (utf16 == NULL)
    return NULL;
  UErrorCode status = U_ZERO_ERROR;
  UNumberFormatter *formatter =
      unumf_openForSkeletonAndLocale(utf16, utf16_length, locale, &status);
  free(utf16);
  if (U_FAILURE(status) || formatter == NULL) {
    if (formatter != NULL)
      unumf_close(formatter);
    wow_icu_fail(error, U_FAILURE(status) ? status : U_INTERNAL_PROGRAM_ERROR,
                 "open duration unit formatter");
    return NULL;
  }
  return formatter;
}

static UChar *copy_formatted_unit(const UFormattedNumber *result,
                                 int32_t *length, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t required = unumf_resultToString(result, NULL, 0, &status);
  if (!wow_icu_preflight(status, error, "size formatted duration unit"))
    return NULL;
  UChar *text = wow_icu_allocate(required, sizeof(UChar), error);
  if (text == NULL)
    return NULL;
  status = U_ZERO_ERROR;
  *length = unumf_resultToString(result, text, required + 1, &status);
  if (U_FAILURE(status)) {
    free(text);
    wow_icu_fail(error, status, "copy formatted duration unit");
    return NULL;
  }
  return text;
}

static UChar *format_unit(const char *locale, const WowIcuDurationPart *part,
                          int32_t width, int32_t *length, WowIcuError *error) {
  UNumberFormatter *formatter = open_unit_formatter(locale, part, width, error);
  if (formatter == NULL)
    return NULL;
  UErrorCode status = U_ZERO_ERROR;
  UFormattedNumber *result = unumf_openResult(&status);
  UChar *text = NULL;
  if (U_SUCCESS(status) && result != NULL) {
    unumf_formatDouble(formatter, part->value, result, &status);
    if (U_SUCCESS(status))
      text = copy_formatted_unit(result, length, error);
    else
      wow_icu_fail(error, status, "format duration unit");
  } else {
    wow_icu_fail(error, U_FAILURE(status) ? status : U_MEMORY_ALLOCATION_ERROR,
                 "open duration unit result");
  }
  if (result != NULL)
    unumf_closeResult(result);
  unumf_close(formatter);
  return text;
}

static UBool join_units(const UListFormatter *formatter,
                        const UChar *const *parts, const int32_t *lengths,
                        int32_t count, WowIcuString *output, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t required = ulistfmt_format(formatter, parts, lengths, count, NULL, 0, &status);
  if (!wow_icu_preflight(status, error, "size duration unit list"))
    return 0;
  UChar *joined = wow_icu_allocate(required, sizeof(UChar), error);
  if (joined == NULL)
    return 0;
  status = U_ZERO_ERROR;
  int32_t length = ulistfmt_format(formatter, parts, lengths, count, joined,
                                  required + 1, &status);
  UBool success = 0;
  if (U_SUCCESS(status))
    success = wow_icu_to_utf8(joined, length, output, error);
  else
    wow_icu_fail(error, status, "format duration unit list");
  free(joined);
  return success;
}

static UBool format_unit_list(const char *locale, const WowIcuDurationPart *parts,
                              int32_t count, int32_t width,
                              WowIcuString *output, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  UListFormatter *formatter = ulistfmt_openForType(locale, ULISTFMT_TYPE_UNITS,
                                                 list_width(width), &status);
  if (U_FAILURE(status) || formatter == NULL) {
    if (formatter != NULL)
      ulistfmt_close(formatter);
    wow_icu_fail(error, U_FAILURE(status) ? status : U_INTERNAL_PROGRAM_ERROR,
                 "open duration unit list formatter");
    return 0;
  }
  UChar *owned[4] = {NULL, NULL, NULL, NULL};
  const UChar *views[4];
  int32_t lengths[4];
  int32_t completed = 0;
  for (; completed < count; ++completed) {
    owned[completed] = format_unit(locale, &parts[completed], width,
                                   &lengths[completed], error);
    if (owned[completed] == NULL)
      break;
    views[completed] = owned[completed];
  }
  UBool success = completed == count &&
      join_units(formatter, views, lengths, count, output, error);
  for (int32_t i = 0; i < count; ++i)
    free(owned[i]);
  ulistfmt_close(formatter);
  return success;
}

int32_t wow_icu_duration_units(const char *locale, int32_t locale_length,
                                const WowIcuDurationPart *parts, int32_t count,
                                int32_t width, WowIcuString *output,
                                WowIcuError *error) {
  output->data = NULL;
  output->length = 0;
  if (count < 1 || count > 4 || width < 0 || width > 2)
    return wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR, "validate duration unit list");
  for (int32_t i = 0; i < count; ++i) {
    const WowIcuDurationPart *part = &parts[i];
    UBool valid_precision = part->fraction_digits == 0 || part->fraction_digits == 3;
    if (!isfinite(part->value) || part->value < 0 || part->unit < 0 ||
        part->unit > 3 || !valid_precision)
      return wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR, "validate duration unit");
  }
  char *parsed_locale = wow_icu_locale(locale, locale_length, error);
  if (parsed_locale == NULL)
    return WOW_ICU_ERROR;
  UBool success = format_unit_list(parsed_locale, parts, count, width, output, error);
  free(parsed_locale);
  return success ? WOW_ICU_OK : WOW_ICU_ERROR;
}
