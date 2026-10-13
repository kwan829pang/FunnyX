/// Corp onboarding models (Client Center `/v1/corp/*`).
library;

class CorpDocument {
  const CorpDocument({
    required this.id,
    required this.docType,
    required this.fileName,
    required this.contentType,
    required this.byteSize,
    required this.status,
    this.gameId,
  });

  final int id;
  final String docType;
  final String fileName;
  final String contentType;
  final int byteSize;
  final String status;
  final int? gameId;

  factory CorpDocument.fromJson(Map<String, dynamic> json) {
    return CorpDocument(
      id: _asInt(json['id']),
      docType: json['doc_type']?.toString() ?? '',
      fileName: json['file_name']?.toString() ?? '',
      contentType: json['content_type']?.toString() ?? '',
      byteSize: _asInt(json['byte_size']),
      status: json['status']?.toString() ?? '',
      gameId: json['game_id'] == null ? null : _asInt(json['game_id']),
    );
  }
}

class CorpProfile {
  const CorpProfile({
    required this.id,
    required this.companyName,
    required this.masterCode,
    required this.masterId,
    required this.status,
    required this.apiEnabled,
    this.contactEmail,
    this.contactName,
    this.rejectReason,
    required this.approvedAt,
    required this.createdAt,
    required this.documents,
  });

  final int id;
  final String companyName;
  final String masterCode;
  final String masterId;
  final String status;
  final bool apiEnabled;
  final String? contactEmail;
  final String? contactName;
  final String? rejectReason;
  final int approvedAt;
  final int createdAt;
  final List<CorpDocument> documents;

  bool get isPending => status == 'pending';
  bool get isActive => status == 'active';
  bool get isRejected => status == 'rejected';

  factory CorpProfile.fromJson(Map<String, dynamic> json) {
    final docs = json['documents'];
    return CorpProfile(
      id: _asInt(json['id']),
      companyName: json['company_name']?.toString() ?? '',
      masterCode: json['master_code']?.toString() ?? '',
      masterId: json['master_id']?.toString() ?? '',
      status: json['status']?.toString() ?? '',
      apiEnabled: json['api_enabled'] == true,
      contactEmail: json['contact_email']?.toString(),
      contactName: json['contact_name']?.toString(),
      rejectReason: json['reject_reason']?.toString(),
      approvedAt: _asInt(json['approved_at']),
      createdAt: _asInt(json['created_at']),
      documents: docs is List
          ? docs
                .whereType<Map>()
                .map((e) => CorpDocument.fromJson(Map<String, dynamic>.from(e)))
                .toList()
          : const [],
    );
  }
}

class CorpGame {
  const CorpGame({
    required this.id,
    required this.corporateUserId,
    required this.gameName,
    required this.gameCode,
    required this.partnerCode,
    required this.partnerGameId,
    required this.status,
    required this.description,
    required this.rejectReason,
    required this.approvedAt,
    required this.createdAt,
    required this.documents,
  });

  final int id;
  final int corporateUserId;
  final String gameName;
  final String gameCode;
  final String partnerCode;
  final String partnerGameId;
  final String status;
  final String description;
  final String rejectReason;
  final int approvedAt;
  final int createdAt;
  final List<CorpDocument> documents;

  factory CorpGame.fromJson(Map<String, dynamic> json) {
    final docs = json['documents'];
    return CorpGame(
      id: _asInt(json['id']),
      corporateUserId: _asInt(json['corporate_user_id']),
      gameName: json['game_name']?.toString() ?? '',
      gameCode: json['game_code']?.toString() ?? '',
      partnerCode: json['partner_code']?.toString() ?? '',
      partnerGameId: json['partner_game_id']?.toString() ?? '',
      status: json['status']?.toString() ?? '',
      description: json['description']?.toString() ?? '',
      rejectReason: json['reject_reason']?.toString() ?? '',
      approvedAt: _asInt(json['approved_at']),
      createdAt: _asInt(json['created_at']),
      documents: docs is List
          ? docs
                .whereType<Map>()
                .map((e) => CorpDocument.fromJson(Map<String, dynamic>.from(e)))
                .toList()
          : const [],
    );
  }
}

class CorpAuthResult {
  const CorpAuthResult({
    required this.profile,
    required this.accessToken,
    required this.expiresAt,
    this.apiKey,
    this.apiSecret,
    this.message,
  });

  final CorpProfile profile;
  final String accessToken;
  final int expiresAt;
  final String? apiKey;
  final String? apiSecret;
  final String? message;
}

int _asInt(dynamic v) {
  if (v is int) return v;
  if (v is num) return v.toInt();
  return int.tryParse(v?.toString() ?? '') ?? 0;
}
