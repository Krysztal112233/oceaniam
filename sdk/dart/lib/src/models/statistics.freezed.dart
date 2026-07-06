// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'statistics.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Overview _$OverviewFromJson(Map<String, dynamic> json) {
  return _Overview.fromJson(json);
}

/// @nodoc
mixin _$Overview {
  @JsonKey(name: 'total_tenants')
  int get totalTenants => throw _privateConstructorUsedError;
  @JsonKey(name: 'total_applications')
  int get totalApplications => throw _privateConstructorUsedError;
  @JsonKey(name: 'total_administrators')
  int get totalAdministrators => throw _privateConstructorUsedError;
  @JsonKey(name: 'total_application_users')
  int get totalApplicationUsers => throw _privateConstructorUsedError;
  @JsonKey(name: 'total_active_secrets')
  int get totalActiveSecrets => throw _privateConstructorUsedError;

  /// Serializes this Overview to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Overview
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $OverviewCopyWith<Overview> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $OverviewCopyWith<$Res> {
  factory $OverviewCopyWith(Overview value, $Res Function(Overview) then) =
      _$OverviewCopyWithImpl<$Res, Overview>;
  @useResult
  $Res call(
      {@JsonKey(name: 'total_tenants') int totalTenants,
      @JsonKey(name: 'total_applications') int totalApplications,
      @JsonKey(name: 'total_administrators') int totalAdministrators,
      @JsonKey(name: 'total_application_users') int totalApplicationUsers,
      @JsonKey(name: 'total_active_secrets') int totalActiveSecrets});
}

/// @nodoc
class _$OverviewCopyWithImpl<$Res, $Val extends Overview>
    implements $OverviewCopyWith<$Res> {
  _$OverviewCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Overview
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? totalTenants = null,
    Object? totalApplications = null,
    Object? totalAdministrators = null,
    Object? totalApplicationUsers = null,
    Object? totalActiveSecrets = null,
  }) {
    return _then(_value.copyWith(
      totalTenants: null == totalTenants
          ? _value.totalTenants
          : totalTenants // ignore: cast_nullable_to_non_nullable
              as int,
      totalApplications: null == totalApplications
          ? _value.totalApplications
          : totalApplications // ignore: cast_nullable_to_non_nullable
              as int,
      totalAdministrators: null == totalAdministrators
          ? _value.totalAdministrators
          : totalAdministrators // ignore: cast_nullable_to_non_nullable
              as int,
      totalApplicationUsers: null == totalApplicationUsers
          ? _value.totalApplicationUsers
          : totalApplicationUsers // ignore: cast_nullable_to_non_nullable
              as int,
      totalActiveSecrets: null == totalActiveSecrets
          ? _value.totalActiveSecrets
          : totalActiveSecrets // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$OverviewImplCopyWith<$Res>
    implements $OverviewCopyWith<$Res> {
  factory _$$OverviewImplCopyWith(
          _$OverviewImpl value, $Res Function(_$OverviewImpl) then) =
      __$$OverviewImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'total_tenants') int totalTenants,
      @JsonKey(name: 'total_applications') int totalApplications,
      @JsonKey(name: 'total_administrators') int totalAdministrators,
      @JsonKey(name: 'total_application_users') int totalApplicationUsers,
      @JsonKey(name: 'total_active_secrets') int totalActiveSecrets});
}

/// @nodoc
class __$$OverviewImplCopyWithImpl<$Res>
    extends _$OverviewCopyWithImpl<$Res, _$OverviewImpl>
    implements _$$OverviewImplCopyWith<$Res> {
  __$$OverviewImplCopyWithImpl(
      _$OverviewImpl _value, $Res Function(_$OverviewImpl) _then)
      : super(_value, _then);

  /// Create a copy of Overview
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? totalTenants = null,
    Object? totalApplications = null,
    Object? totalAdministrators = null,
    Object? totalApplicationUsers = null,
    Object? totalActiveSecrets = null,
  }) {
    return _then(_$OverviewImpl(
      totalTenants: null == totalTenants
          ? _value.totalTenants
          : totalTenants // ignore: cast_nullable_to_non_nullable
              as int,
      totalApplications: null == totalApplications
          ? _value.totalApplications
          : totalApplications // ignore: cast_nullable_to_non_nullable
              as int,
      totalAdministrators: null == totalAdministrators
          ? _value.totalAdministrators
          : totalAdministrators // ignore: cast_nullable_to_non_nullable
              as int,
      totalApplicationUsers: null == totalApplicationUsers
          ? _value.totalApplicationUsers
          : totalApplicationUsers // ignore: cast_nullable_to_non_nullable
              as int,
      totalActiveSecrets: null == totalActiveSecrets
          ? _value.totalActiveSecrets
          : totalActiveSecrets // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$OverviewImpl implements _Overview {
  const _$OverviewImpl(
      {@JsonKey(name: 'total_tenants') required this.totalTenants,
      @JsonKey(name: 'total_applications') required this.totalApplications,
      @JsonKey(name: 'total_administrators') required this.totalAdministrators,
      @JsonKey(name: 'total_application_users')
      required this.totalApplicationUsers,
      @JsonKey(name: 'total_active_secrets') required this.totalActiveSecrets});

  factory _$OverviewImpl.fromJson(Map<String, dynamic> json) =>
      _$$OverviewImplFromJson(json);

  @override
  @JsonKey(name: 'total_tenants')
  final int totalTenants;
  @override
  @JsonKey(name: 'total_applications')
  final int totalApplications;
  @override
  @JsonKey(name: 'total_administrators')
  final int totalAdministrators;
  @override
  @JsonKey(name: 'total_application_users')
  final int totalApplicationUsers;
  @override
  @JsonKey(name: 'total_active_secrets')
  final int totalActiveSecrets;

  @override
  String toString() {
    return 'Overview(totalTenants: $totalTenants, totalApplications: $totalApplications, totalAdministrators: $totalAdministrators, totalApplicationUsers: $totalApplicationUsers, totalActiveSecrets: $totalActiveSecrets)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$OverviewImpl &&
            (identical(other.totalTenants, totalTenants) ||
                other.totalTenants == totalTenants) &&
            (identical(other.totalApplications, totalApplications) ||
                other.totalApplications == totalApplications) &&
            (identical(other.totalAdministrators, totalAdministrators) ||
                other.totalAdministrators == totalAdministrators) &&
            (identical(other.totalApplicationUsers, totalApplicationUsers) ||
                other.totalApplicationUsers == totalApplicationUsers) &&
            (identical(other.totalActiveSecrets, totalActiveSecrets) ||
                other.totalActiveSecrets == totalActiveSecrets));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, totalTenants, totalApplications,
      totalAdministrators, totalApplicationUsers, totalActiveSecrets);

  /// Create a copy of Overview
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$OverviewImplCopyWith<_$OverviewImpl> get copyWith =>
      __$$OverviewImplCopyWithImpl<_$OverviewImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$OverviewImplToJson(
      this,
    );
  }
}

abstract class _Overview implements Overview {
  const factory _Overview(
      {@JsonKey(name: 'total_tenants') required final int totalTenants,
      @JsonKey(name: 'total_applications') required final int totalApplications,
      @JsonKey(name: 'total_administrators')
      required final int totalAdministrators,
      @JsonKey(name: 'total_application_users')
      required final int totalApplicationUsers,
      @JsonKey(name: 'total_active_secrets')
      required final int totalActiveSecrets}) = _$OverviewImpl;

  factory _Overview.fromJson(Map<String, dynamic> json) =
      _$OverviewImpl.fromJson;

  @override
  @JsonKey(name: 'total_tenants')
  int get totalTenants;
  @override
  @JsonKey(name: 'total_applications')
  int get totalApplications;
  @override
  @JsonKey(name: 'total_administrators')
  int get totalAdministrators;
  @override
  @JsonKey(name: 'total_application_users')
  int get totalApplicationUsers;
  @override
  @JsonKey(name: 'total_active_secrets')
  int get totalActiveSecrets;

  /// Create a copy of Overview
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$OverviewImplCopyWith<_$OverviewImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

TrendDataPoint _$TrendDataPointFromJson(Map<String, dynamic> json) {
  return _TrendDataPoint.fromJson(json);
}

/// @nodoc
mixin _$TrendDataPoint {
  DateTime get bucket => throw _privateConstructorUsedError;
  int get count => throw _privateConstructorUsedError;

  /// Serializes this TrendDataPoint to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of TrendDataPoint
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $TrendDataPointCopyWith<TrendDataPoint> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $TrendDataPointCopyWith<$Res> {
  factory $TrendDataPointCopyWith(
          TrendDataPoint value, $Res Function(TrendDataPoint) then) =
      _$TrendDataPointCopyWithImpl<$Res, TrendDataPoint>;
  @useResult
  $Res call({DateTime bucket, int count});
}

/// @nodoc
class _$TrendDataPointCopyWithImpl<$Res, $Val extends TrendDataPoint>
    implements $TrendDataPointCopyWith<$Res> {
  _$TrendDataPointCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of TrendDataPoint
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bucket = null,
    Object? count = null,
  }) {
    return _then(_value.copyWith(
      bucket: null == bucket
          ? _value.bucket
          : bucket // ignore: cast_nullable_to_non_nullable
              as DateTime,
      count: null == count
          ? _value.count
          : count // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$TrendDataPointImplCopyWith<$Res>
    implements $TrendDataPointCopyWith<$Res> {
  factory _$$TrendDataPointImplCopyWith(_$TrendDataPointImpl value,
          $Res Function(_$TrendDataPointImpl) then) =
      __$$TrendDataPointImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({DateTime bucket, int count});
}

/// @nodoc
class __$$TrendDataPointImplCopyWithImpl<$Res>
    extends _$TrendDataPointCopyWithImpl<$Res, _$TrendDataPointImpl>
    implements _$$TrendDataPointImplCopyWith<$Res> {
  __$$TrendDataPointImplCopyWithImpl(
      _$TrendDataPointImpl _value, $Res Function(_$TrendDataPointImpl) _then)
      : super(_value, _then);

  /// Create a copy of TrendDataPoint
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bucket = null,
    Object? count = null,
  }) {
    return _then(_$TrendDataPointImpl(
      bucket: null == bucket
          ? _value.bucket
          : bucket // ignore: cast_nullable_to_non_nullable
              as DateTime,
      count: null == count
          ? _value.count
          : count // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$TrendDataPointImpl implements _TrendDataPoint {
  const _$TrendDataPointImpl({required this.bucket, required this.count});

  factory _$TrendDataPointImpl.fromJson(Map<String, dynamic> json) =>
      _$$TrendDataPointImplFromJson(json);

  @override
  final DateTime bucket;
  @override
  final int count;

  @override
  String toString() {
    return 'TrendDataPoint(bucket: $bucket, count: $count)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$TrendDataPointImpl &&
            (identical(other.bucket, bucket) || other.bucket == bucket) &&
            (identical(other.count, count) || other.count == count));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, bucket, count);

  /// Create a copy of TrendDataPoint
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$TrendDataPointImplCopyWith<_$TrendDataPointImpl> get copyWith =>
      __$$TrendDataPointImplCopyWithImpl<_$TrendDataPointImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$TrendDataPointImplToJson(
      this,
    );
  }
}

abstract class _TrendDataPoint implements TrendDataPoint {
  const factory _TrendDataPoint(
      {required final DateTime bucket,
      required final int count}) = _$TrendDataPointImpl;

  factory _TrendDataPoint.fromJson(Map<String, dynamic> json) =
      _$TrendDataPointImpl.fromJson;

  @override
  DateTime get bucket;
  @override
  int get count;

  /// Create a copy of TrendDataPoint
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$TrendDataPointImplCopyWith<_$TrendDataPointImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

PlatformTrends _$PlatformTrendsFromJson(Map<String, dynamic> json) {
  return _PlatformTrends.fromJson(json);
}

/// @nodoc
mixin _$PlatformTrends {
  String get granularity => throw _privateConstructorUsedError;
  int get range => throw _privateConstructorUsedError;
  List<TrendDataPoint> get tenants => throw _privateConstructorUsedError;
  List<TrendDataPoint> get applications => throw _privateConstructorUsedError;
  List<TrendDataPoint> get users => throw _privateConstructorUsedError;
  List<TrendDataPoint> get administrators => throw _privateConstructorUsedError;

  /// Serializes this PlatformTrends to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PlatformTrends
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PlatformTrendsCopyWith<PlatformTrends> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PlatformTrendsCopyWith<$Res> {
  factory $PlatformTrendsCopyWith(
          PlatformTrends value, $Res Function(PlatformTrends) then) =
      _$PlatformTrendsCopyWithImpl<$Res, PlatformTrends>;
  @useResult
  $Res call(
      {String granularity,
      int range,
      List<TrendDataPoint> tenants,
      List<TrendDataPoint> applications,
      List<TrendDataPoint> users,
      List<TrendDataPoint> administrators});
}

/// @nodoc
class _$PlatformTrendsCopyWithImpl<$Res, $Val extends PlatformTrends>
    implements $PlatformTrendsCopyWith<$Res> {
  _$PlatformTrendsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PlatformTrends
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? granularity = null,
    Object? range = null,
    Object? tenants = null,
    Object? applications = null,
    Object? users = null,
    Object? administrators = null,
  }) {
    return _then(_value.copyWith(
      granularity: null == granularity
          ? _value.granularity
          : granularity // ignore: cast_nullable_to_non_nullable
              as String,
      range: null == range
          ? _value.range
          : range // ignore: cast_nullable_to_non_nullable
              as int,
      tenants: null == tenants
          ? _value.tenants
          : tenants // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
      applications: null == applications
          ? _value.applications
          : applications // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
      users: null == users
          ? _value.users
          : users // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
      administrators: null == administrators
          ? _value.administrators
          : administrators // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$PlatformTrendsImplCopyWith<$Res>
    implements $PlatformTrendsCopyWith<$Res> {
  factory _$$PlatformTrendsImplCopyWith(_$PlatformTrendsImpl value,
          $Res Function(_$PlatformTrendsImpl) then) =
      __$$PlatformTrendsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String granularity,
      int range,
      List<TrendDataPoint> tenants,
      List<TrendDataPoint> applications,
      List<TrendDataPoint> users,
      List<TrendDataPoint> administrators});
}

/// @nodoc
class __$$PlatformTrendsImplCopyWithImpl<$Res>
    extends _$PlatformTrendsCopyWithImpl<$Res, _$PlatformTrendsImpl>
    implements _$$PlatformTrendsImplCopyWith<$Res> {
  __$$PlatformTrendsImplCopyWithImpl(
      _$PlatformTrendsImpl _value, $Res Function(_$PlatformTrendsImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlatformTrends
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? granularity = null,
    Object? range = null,
    Object? tenants = null,
    Object? applications = null,
    Object? users = null,
    Object? administrators = null,
  }) {
    return _then(_$PlatformTrendsImpl(
      granularity: null == granularity
          ? _value.granularity
          : granularity // ignore: cast_nullable_to_non_nullable
              as String,
      range: null == range
          ? _value.range
          : range // ignore: cast_nullable_to_non_nullable
              as int,
      tenants: null == tenants
          ? _value._tenants
          : tenants // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
      applications: null == applications
          ? _value._applications
          : applications // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
      users: null == users
          ? _value._users
          : users // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
      administrators: null == administrators
          ? _value._administrators
          : administrators // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$PlatformTrendsImpl implements _PlatformTrends {
  const _$PlatformTrendsImpl(
      {required this.granularity,
      required this.range,
      required final List<TrendDataPoint> tenants,
      required final List<TrendDataPoint> applications,
      required final List<TrendDataPoint> users,
      required final List<TrendDataPoint> administrators})
      : _tenants = tenants,
        _applications = applications,
        _users = users,
        _administrators = administrators;

  factory _$PlatformTrendsImpl.fromJson(Map<String, dynamic> json) =>
      _$$PlatformTrendsImplFromJson(json);

  @override
  final String granularity;
  @override
  final int range;
  final List<TrendDataPoint> _tenants;
  @override
  List<TrendDataPoint> get tenants {
    if (_tenants is EqualUnmodifiableListView) return _tenants;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_tenants);
  }

  final List<TrendDataPoint> _applications;
  @override
  List<TrendDataPoint> get applications {
    if (_applications is EqualUnmodifiableListView) return _applications;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_applications);
  }

  final List<TrendDataPoint> _users;
  @override
  List<TrendDataPoint> get users {
    if (_users is EqualUnmodifiableListView) return _users;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_users);
  }

  final List<TrendDataPoint> _administrators;
  @override
  List<TrendDataPoint> get administrators {
    if (_administrators is EqualUnmodifiableListView) return _administrators;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_administrators);
  }

  @override
  String toString() {
    return 'PlatformTrends(granularity: $granularity, range: $range, tenants: $tenants, applications: $applications, users: $users, administrators: $administrators)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlatformTrendsImpl &&
            (identical(other.granularity, granularity) ||
                other.granularity == granularity) &&
            (identical(other.range, range) || other.range == range) &&
            const DeepCollectionEquality().equals(other._tenants, _tenants) &&
            const DeepCollectionEquality()
                .equals(other._applications, _applications) &&
            const DeepCollectionEquality().equals(other._users, _users) &&
            const DeepCollectionEquality()
                .equals(other._administrators, _administrators));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      granularity,
      range,
      const DeepCollectionEquality().hash(_tenants),
      const DeepCollectionEquality().hash(_applications),
      const DeepCollectionEquality().hash(_users),
      const DeepCollectionEquality().hash(_administrators));

  /// Create a copy of PlatformTrends
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlatformTrendsImplCopyWith<_$PlatformTrendsImpl> get copyWith =>
      __$$PlatformTrendsImplCopyWithImpl<_$PlatformTrendsImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PlatformTrendsImplToJson(
      this,
    );
  }
}

abstract class _PlatformTrends implements PlatformTrends {
  const factory _PlatformTrends(
          {required final String granularity,
          required final int range,
          required final List<TrendDataPoint> tenants,
          required final List<TrendDataPoint> applications,
          required final List<TrendDataPoint> users,
          required final List<TrendDataPoint> administrators}) =
      _$PlatformTrendsImpl;

  factory _PlatformTrends.fromJson(Map<String, dynamic> json) =
      _$PlatformTrendsImpl.fromJson;

  @override
  String get granularity;
  @override
  int get range;
  @override
  List<TrendDataPoint> get tenants;
  @override
  List<TrendDataPoint> get applications;
  @override
  List<TrendDataPoint> get users;
  @override
  List<TrendDataPoint> get administrators;

  /// Create a copy of PlatformTrends
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlatformTrendsImplCopyWith<_$PlatformTrendsImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

ApplicationStatistics _$ApplicationStatisticsFromJson(
    Map<String, dynamic> json) {
  return _ApplicationStatistics.fromJson(json);
}

/// @nodoc
mixin _$ApplicationStatistics {
  @JsonKey(name: 'total_users')
  int get totalUsers => throw _privateConstructorUsedError;
  @JsonKey(name: 'active_users')
  int get activeUsers => throw _privateConstructorUsedError;

  /// Serializes this ApplicationStatistics to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationStatistics
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationStatisticsCopyWith<ApplicationStatistics> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationStatisticsCopyWith<$Res> {
  factory $ApplicationStatisticsCopyWith(ApplicationStatistics value,
          $Res Function(ApplicationStatistics) then) =
      _$ApplicationStatisticsCopyWithImpl<$Res, ApplicationStatistics>;
  @useResult
  $Res call(
      {@JsonKey(name: 'total_users') int totalUsers,
      @JsonKey(name: 'active_users') int activeUsers});
}

/// @nodoc
class _$ApplicationStatisticsCopyWithImpl<$Res,
        $Val extends ApplicationStatistics>
    implements $ApplicationStatisticsCopyWith<$Res> {
  _$ApplicationStatisticsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationStatistics
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? totalUsers = null,
    Object? activeUsers = null,
  }) {
    return _then(_value.copyWith(
      totalUsers: null == totalUsers
          ? _value.totalUsers
          : totalUsers // ignore: cast_nullable_to_non_nullable
              as int,
      activeUsers: null == activeUsers
          ? _value.activeUsers
          : activeUsers // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationStatisticsImplCopyWith<$Res>
    implements $ApplicationStatisticsCopyWith<$Res> {
  factory _$$ApplicationStatisticsImplCopyWith(
          _$ApplicationStatisticsImpl value,
          $Res Function(_$ApplicationStatisticsImpl) then) =
      __$$ApplicationStatisticsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'total_users') int totalUsers,
      @JsonKey(name: 'active_users') int activeUsers});
}

/// @nodoc
class __$$ApplicationStatisticsImplCopyWithImpl<$Res>
    extends _$ApplicationStatisticsCopyWithImpl<$Res,
        _$ApplicationStatisticsImpl>
    implements _$$ApplicationStatisticsImplCopyWith<$Res> {
  __$$ApplicationStatisticsImplCopyWithImpl(_$ApplicationStatisticsImpl _value,
      $Res Function(_$ApplicationStatisticsImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationStatistics
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? totalUsers = null,
    Object? activeUsers = null,
  }) {
    return _then(_$ApplicationStatisticsImpl(
      totalUsers: null == totalUsers
          ? _value.totalUsers
          : totalUsers // ignore: cast_nullable_to_non_nullable
              as int,
      activeUsers: null == activeUsers
          ? _value.activeUsers
          : activeUsers // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationStatisticsImpl implements _ApplicationStatistics {
  const _$ApplicationStatisticsImpl(
      {@JsonKey(name: 'total_users') required this.totalUsers,
      @JsonKey(name: 'active_users') required this.activeUsers});

  factory _$ApplicationStatisticsImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationStatisticsImplFromJson(json);

  @override
  @JsonKey(name: 'total_users')
  final int totalUsers;
  @override
  @JsonKey(name: 'active_users')
  final int activeUsers;

  @override
  String toString() {
    return 'ApplicationStatistics(totalUsers: $totalUsers, activeUsers: $activeUsers)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationStatisticsImpl &&
            (identical(other.totalUsers, totalUsers) ||
                other.totalUsers == totalUsers) &&
            (identical(other.activeUsers, activeUsers) ||
                other.activeUsers == activeUsers));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, totalUsers, activeUsers);

  /// Create a copy of ApplicationStatistics
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationStatisticsImplCopyWith<_$ApplicationStatisticsImpl>
      get copyWith => __$$ApplicationStatisticsImplCopyWithImpl<
          _$ApplicationStatisticsImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationStatisticsImplToJson(
      this,
    );
  }
}

abstract class _ApplicationStatistics implements ApplicationStatistics {
  const factory _ApplicationStatistics(
          {@JsonKey(name: 'total_users') required final int totalUsers,
          @JsonKey(name: 'active_users') required final int activeUsers}) =
      _$ApplicationStatisticsImpl;

  factory _ApplicationStatistics.fromJson(Map<String, dynamic> json) =
      _$ApplicationStatisticsImpl.fromJson;

  @override
  @JsonKey(name: 'total_users')
  int get totalUsers;
  @override
  @JsonKey(name: 'active_users')
  int get activeUsers;

  /// Create a copy of ApplicationStatistics
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationStatisticsImplCopyWith<_$ApplicationStatisticsImpl>
      get copyWith => throw _privateConstructorUsedError;
}

ApplicationTrends _$ApplicationTrendsFromJson(Map<String, dynamic> json) {
  return _ApplicationTrends.fromJson(json);
}

/// @nodoc
mixin _$ApplicationTrends {
  String get granularity => throw _privateConstructorUsedError;
  int get range => throw _privateConstructorUsedError;
  @JsonKey(name: 'new_users')
  List<TrendDataPoint> get newUsers => throw _privateConstructorUsedError;

  /// Serializes this ApplicationTrends to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationTrends
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationTrendsCopyWith<ApplicationTrends> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationTrendsCopyWith<$Res> {
  factory $ApplicationTrendsCopyWith(
          ApplicationTrends value, $Res Function(ApplicationTrends) then) =
      _$ApplicationTrendsCopyWithImpl<$Res, ApplicationTrends>;
  @useResult
  $Res call(
      {String granularity,
      int range,
      @JsonKey(name: 'new_users') List<TrendDataPoint> newUsers});
}

/// @nodoc
class _$ApplicationTrendsCopyWithImpl<$Res, $Val extends ApplicationTrends>
    implements $ApplicationTrendsCopyWith<$Res> {
  _$ApplicationTrendsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationTrends
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? granularity = null,
    Object? range = null,
    Object? newUsers = null,
  }) {
    return _then(_value.copyWith(
      granularity: null == granularity
          ? _value.granularity
          : granularity // ignore: cast_nullable_to_non_nullable
              as String,
      range: null == range
          ? _value.range
          : range // ignore: cast_nullable_to_non_nullable
              as int,
      newUsers: null == newUsers
          ? _value.newUsers
          : newUsers // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationTrendsImplCopyWith<$Res>
    implements $ApplicationTrendsCopyWith<$Res> {
  factory _$$ApplicationTrendsImplCopyWith(_$ApplicationTrendsImpl value,
          $Res Function(_$ApplicationTrendsImpl) then) =
      __$$ApplicationTrendsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String granularity,
      int range,
      @JsonKey(name: 'new_users') List<TrendDataPoint> newUsers});
}

/// @nodoc
class __$$ApplicationTrendsImplCopyWithImpl<$Res>
    extends _$ApplicationTrendsCopyWithImpl<$Res, _$ApplicationTrendsImpl>
    implements _$$ApplicationTrendsImplCopyWith<$Res> {
  __$$ApplicationTrendsImplCopyWithImpl(_$ApplicationTrendsImpl _value,
      $Res Function(_$ApplicationTrendsImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationTrends
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? granularity = null,
    Object? range = null,
    Object? newUsers = null,
  }) {
    return _then(_$ApplicationTrendsImpl(
      granularity: null == granularity
          ? _value.granularity
          : granularity // ignore: cast_nullable_to_non_nullable
              as String,
      range: null == range
          ? _value.range
          : range // ignore: cast_nullable_to_non_nullable
              as int,
      newUsers: null == newUsers
          ? _value._newUsers
          : newUsers // ignore: cast_nullable_to_non_nullable
              as List<TrendDataPoint>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationTrendsImpl implements _ApplicationTrends {
  const _$ApplicationTrendsImpl(
      {required this.granularity,
      required this.range,
      @JsonKey(name: 'new_users') required final List<TrendDataPoint> newUsers})
      : _newUsers = newUsers;

  factory _$ApplicationTrendsImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationTrendsImplFromJson(json);

  @override
  final String granularity;
  @override
  final int range;
  final List<TrendDataPoint> _newUsers;
  @override
  @JsonKey(name: 'new_users')
  List<TrendDataPoint> get newUsers {
    if (_newUsers is EqualUnmodifiableListView) return _newUsers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_newUsers);
  }

  @override
  String toString() {
    return 'ApplicationTrends(granularity: $granularity, range: $range, newUsers: $newUsers)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationTrendsImpl &&
            (identical(other.granularity, granularity) ||
                other.granularity == granularity) &&
            (identical(other.range, range) || other.range == range) &&
            const DeepCollectionEquality().equals(other._newUsers, _newUsers));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, granularity, range,
      const DeepCollectionEquality().hash(_newUsers));

  /// Create a copy of ApplicationTrends
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationTrendsImplCopyWith<_$ApplicationTrendsImpl> get copyWith =>
      __$$ApplicationTrendsImplCopyWithImpl<_$ApplicationTrendsImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationTrendsImplToJson(
      this,
    );
  }
}

abstract class _ApplicationTrends implements ApplicationTrends {
  const factory _ApplicationTrends(
      {required final String granularity,
      required final int range,
      @JsonKey(name: 'new_users')
      required final List<TrendDataPoint> newUsers}) = _$ApplicationTrendsImpl;

  factory _ApplicationTrends.fromJson(Map<String, dynamic> json) =
      _$ApplicationTrendsImpl.fromJson;

  @override
  String get granularity;
  @override
  int get range;
  @override
  @JsonKey(name: 'new_users')
  List<TrendDataPoint> get newUsers;

  /// Create a copy of ApplicationTrends
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationTrendsImplCopyWith<_$ApplicationTrendsImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
