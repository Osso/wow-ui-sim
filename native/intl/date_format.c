#include "bridge.h"

#include <math.h>
#include <stdlib.h>
#include <unicode/ucal.h>
#include <unicode/udat.h>

static UDateFormatStyle date_style(int32_t style) {
  static const UDateFormatStyle styles[] = {
      UDAT_NONE, UDAT_SHORT, UDAT_MEDIUM, UDAT_LONG, UDAT_FULL};
  return styles[style];
}

static UBool validate_zone(const UChar *zone, int32_t length,
                           WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  UBool system = 0;
  int32_t required = ucal_getCanonicalTimeZoneID(zone, length, NULL, 0,
                                                &system, &status);
  if (!wow_icu_preflight(status, error, "validate time zone"))
    return 0;
  UChar *canonical = wow_icu_allocate(required, sizeof(UChar), error);
  if (canonical == NULL)
    return 0;
  status = U_ZERO_ERROR;
  ucal_getCanonicalTimeZoneID(zone, length, canonical, required + 1,
                              &system, &status);
  free(canonical);
  if (U_FAILURE(status)) {
    wow_icu_fail(error, status, "canonicalize time zone");
    return 0;
  }
  return 1;
}

static UBool format_date(const UDateFormat *formatter, double milliseconds,
                         WowIcuString *output, WowIcuError *error) {
  UErrorCode status = U_ZERO_ERROR;
  int32_t required = udat_format(formatter, milliseconds, NULL, 0, NULL, &status);
  if (!wow_icu_preflight(status, error, "size formatted date"))
    return 0;
  UChar *text = wow_icu_allocate(required, sizeof(UChar), error);
  if (text == NULL)
    return 0;
  status = U_ZERO_ERROR;
  int32_t length = udat_format(formatter, milliseconds, text, required + 1,
                               NULL, &status);
  if (U_FAILURE(status)) {
    free(text);
    wow_icu_fail(error, status, "format date");
    return 0;
  }
  UBool success = wow_icu_to_utf8(text, length, output, error);
  free(text);
  return success;
}

static int32_t format_in_zone(const char *locale, const UChar *zone,
                              int32_t zone_length, double milliseconds,
                              int32_t date, int32_t time,
                              WowIcuString *output, WowIcuError *error) {
  if (!validate_zone(zone, zone_length, error))
    return WOW_ICU_ERROR;
  if (date == 0 && time == 0) {
    const UChar empty[] = {0};
    return wow_icu_to_utf8(empty, 0, output, error) ? WOW_ICU_OK : WOW_ICU_ERROR;
  }
  UErrorCode status = U_ZERO_ERROR;
  UDateFormat *formatter = udat_open(date_style(time), date_style(date), locale,
                                    zone, zone_length, NULL, 0, &status);
  if (U_FAILURE(status) || formatter == NULL) {
    if (formatter != NULL)
      udat_close(formatter);
    return wow_icu_fail(error, U_FAILURE(status) ? status : U_MEMORY_ALLOCATION_ERROR,
                         "open date formatter");
  }
  UBool success = format_date(formatter, milliseconds, output, error);
  udat_close(formatter);
  return success ? WOW_ICU_OK : WOW_ICU_ERROR;
}

int32_t wow_icu_format_date_time(const char *locale, int32_t locale_length,
                                 double milliseconds, int32_t date_style_value,
                                 int32_t time_style_value, const uint8_t *zone,
                                 int32_t zone_length, WowIcuString *output,
                                 WowIcuError *error) {
  if (!isfinite(milliseconds) || date_style_value < 0 || date_style_value > 4 ||
      time_style_value < 0 || time_style_value > 4)
    return wow_icu_fail(error, U_ILLEGAL_ARGUMENT_ERROR, "validate date arguments");
  char *parsed_locale = wow_icu_locale(locale, locale_length, error);
  if (parsed_locale == NULL)
    return WOW_ICU_ERROR;
  int32_t utf16_length = 0;
  UChar *utf16_zone = wow_icu_to_utf16(zone, zone_length, &utf16_length, error);
  if (utf16_zone == NULL) {
    free(parsed_locale);
    return WOW_ICU_ERROR;
  }
  int32_t result = format_in_zone(parsed_locale, utf16_zone, utf16_length,
                                  milliseconds, date_style_value, time_style_value,
                                  output, error);
  free(utf16_zone);
  free(parsed_locale);
  return result;
}
