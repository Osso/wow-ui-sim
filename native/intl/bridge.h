#ifndef WOW_INTL_BRIDGE_H
#define WOW_INTL_BRIDGE_H

#include <stddef.h>
#include <stdint.h>
#include <unicode/utypes.h>

#if U_ICU_VERSION_MAJOR_NUM < 72
#error "The PTR ICU4C bridge requires ICU >= 72"
#endif

/* Only these fixed-width types and ordinary C functions cross into Rust. */
typedef struct {
  uint8_t *data;
  int32_t length;
} WowIcuString;

typedef struct {
  int32_t code;
  const char *operation; /* Static storage; never freed by the caller. */
} WowIcuError;

enum { WOW_ICU_OK = 0, WOW_ICU_NO_MATCH = 1, WOW_ICU_ERROR = 2 };
enum { WOW_DECIMAL = 0, WOW_INTEGER = 1, WOW_PERCENT = 2, WOW_CURRENCY = 3 };

int32_t wow_icu_format(const char *locale, int32_t locale_length, int32_t style,
                       double value, const char *currency, WowIcuString *output,
                       WowIcuError *error);
int32_t wow_icu_parse(const char *locale, int32_t locale_length, int32_t style,
                      const uint8_t *text, int32_t text_length,
                      int32_t with_currency, double *value, uint8_t currency[4],
                      WowIcuError *error);
int32_t wow_icu_currency_name(const char *locale, int32_t locale_length,
                              const char *currency, int32_t style,
                              WowIcuString *output, WowIcuError *error);
int32_t wow_icu_currency_fraction_digits(const char *currency, int32_t *digits,
                                         WowIcuError *error);
int32_t wow_icu_format_date_time(const char *locale, int32_t locale_length,
                                 double milliseconds, int32_t date_style,
                                 int32_t time_style, const uint8_t *zone,
                                 int32_t zone_length, WowIcuString *output,
                                 WowIcuError *error);
int32_t wow_icu_display_name(const char *target, int32_t target_length,
                              const char *display, int32_t display_length,
                              WowIcuString *output, WowIcuError *error);
int32_t wow_icu_transliterate(const uint8_t *text, int32_t text_length,
                               const uint8_t *id, int32_t id_length,
                               WowIcuString *output, WowIcuError *error);
void wow_icu_string_free(uint8_t *data);
const char *wow_icu_error_name(int32_t code);
void wow_icu_version(uint8_t version[4]);

/* Internal conversion helpers; buffers always have room for a terminator. */
void *wow_icu_allocate(int32_t length, size_t width, WowIcuError *error);
int32_t wow_icu_fail(WowIcuError *error, UErrorCode code,
                     const char *operation);
UBool wow_icu_preflight(UErrorCode status, WowIcuError *error,
                        const char *operation);
UChar *wow_icu_to_utf16(const uint8_t *text, int32_t length,
                        int32_t *out_length, WowIcuError *error);
UBool wow_icu_to_utf8(const UChar *text, int32_t length, WowIcuString *output,
                      WowIcuError *error);
char *wow_icu_locale(const char *locale, int32_t length, WowIcuError *error);

#endif
