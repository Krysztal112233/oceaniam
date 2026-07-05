import 'package:freezed_annotation/freezed_annotation.dart';

part 'pagination.freezed.dart';
part 'pagination.g.dart';

@freezed
class PageInfo with _$PageInfo {
  const factory PageInfo({
    @JsonKey(name: 'has_next') required bool hasNext,
    required int total,
  }) = _PageInfo;

  factory PageInfo.fromJson(Map<String, dynamic> json) =>
      _$PageInfoFromJson(json);
}

@freezed
class PagedResponse<T> with _$PagedResponse<T> {
  const factory PagedResponse({
    required List<T> items,
    required PageInfo pageInfo,
  }) = _PagedResponse<T>;
}
