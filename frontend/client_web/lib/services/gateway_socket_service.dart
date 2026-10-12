import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import '../controllers/shop_orders_controller.dart';
import '../controllers/wallet_controller.dart';

/// Session WebSocket to Gateway `/v1/ws` for Message Center push fan-out.
class GatewaySocketService extends GetxService {
  GatewaySocketService(this.config);

  final AppConfig config;

  WebSocketChannel? _channel;
  StreamSubscription<dynamic>? _sub;
  Worker? _tokenWorker;

  SessionController get _session => Get.find<SessionController>();

  String get _wsUrl {
    final base = config.apiBaseUrl.trim().replaceAll(RegExp(r'/$'), '');
    if (base.startsWith('https://')) {
      return '${base.replaceFirst('https://', 'wss://')}/v1/ws';
    }
    if (base.startsWith('http://')) {
      return '${base.replaceFirst('http://', 'ws://')}/v1/ws';
    }
    return 'ws://127.0.0.1:8080/v1/ws';
  }

  @override
  void onInit() {
    super.onInit();
    _tokenWorker = ever<String?>(_session.tokenRx, (token) {
      if (token == null || token.isEmpty) {
        disconnect();
      } else {
        connect(token);
      }
    });
    final existing = _session.token;
    if (existing != null && existing.isNotEmpty) {
      connect(existing);
    }
  }

  @override
  void onClose() {
    _tokenWorker?.dispose();
    disconnect();
    super.onClose();
  }

  void connect(String accessToken) {
    disconnect();
    try {
      final channel = WebSocketChannel.connect(Uri.parse(_wsUrl));
      _channel = channel;
      channel.sink.add(jsonEncode({'access_token': accessToken}));
      _sub = channel.stream.listen(
        _onMessage,
        onError: (_) => disconnect(),
        onDone: () => disconnect(),
        cancelOnError: true,
      );
    } catch (_) {
      // Best-effort; HTTP poll remains fallback.
    }
  }

  void disconnect() {
    _sub?.cancel();
    _sub = null;
    _channel?.sink.close();
    _channel = null;
  }

  void _onMessage(dynamic raw) {
    final text = raw is String ? raw : raw?.toString() ?? '';
    if (text.isEmpty) return;
    Map<String, dynamic>? map;
    try {
      final decoded = jsonDecode(text);
      if (decoded is Map) {
        map = Map<String, dynamic>.from(decoded);
      }
    } catch (_) {
      return;
    }
    if (map == null) return;
    if (map['ok'] == true) return;
    if (map['ok'] == false) return;
    if (map['type']?.toString() != 'notification') return;

    final title = map['title']?.toString() ?? 'Notice';
    final body = map['body']?.toString() ?? map['event_type']?.toString() ?? '';
    toastification.show(
      title: Text(title),
      description: body.isEmpty ? null : Text(body),
      type: ToastificationType.info,
      autoCloseDuration: const Duration(seconds: 4),
    );

    final source = map['source_type']?.toString() ?? '';
    if (Get.isRegistered<WalletController>()) {
      if (source.contains('deposit') ||
          source.contains('shop') ||
          source.contains('corp_token') ||
          source.contains('marketplace')) {
        unawaited(Get.find<WalletController>().reload());
      }
    }
    if (Get.isRegistered<ShopOrdersController>() &&
        (source.contains('shop') || source.contains('corp_token'))) {
      unawaited(Get.find<ShopOrdersController>().loadOrders());
    }
  }
}
