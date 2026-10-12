import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

class WalletController extends GetxController {
  final loading = false.obs;
  final busy = false.obs;
  final error = RxnString();
  final profile = Rxn<ClientProfile>();
  final wallets = <ClientWalletBalance>[].obs;
  final transactions = <ClientMoneyTxn>[].obs;
  final gameAccounts = <GameAccountBinding>[].obs;

  WalletDataProvider get _wallet => Get.find<WalletDataProvider>();
  ShopDataProvider get _shop => Get.find<ShopDataProvider>();

  @override
  void onInit() {
    super.onInit();
    reload();
  }

  Future<void> reload() async {
    loading.value = true;
    error.value = null;
    try {
      final results = await Future.wait([
        _wallet.getProfile(),
        _wallet.listWallets(),
        _wallet.listTransactions(),
        _shop.listGameAccounts(),
      ]);
      profile.value = results[0] as ClientProfile;
      wallets.assignAll(results[1] as List<ClientWalletBalance>);
      transactions.assignAll(results[2] as List<ClientMoneyTxn>);
      gameAccounts.assignAll(results[3] as List<GameAccountBinding>);
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
    } finally {
      loading.value = false;
    }
  }

  /// PLT available across wallets (home summary).
  double get pltAvailable {
    var sum = 0.0;
    for (final w in wallets) {
      if (w.gameCoin.toUpperCase() == 'PLT') {
        sum += w.available;
      }
    }
    return sum;
  }

  Future<bool> deposit({
    required int gameAccountId,
    required String gameCoin,
    required String amountText,
  }) async {
    return _submit(
      deposit: true,
      gameAccountId: gameAccountId,
      gameCoin: gameCoin,
      amountText: amountText,
    );
  }

  Future<bool> withdraw({
    required int gameAccountId,
    required String gameCoin,
    required String amountText,
  }) async {
    return _submit(
      deposit: false,
      gameAccountId: gameAccountId,
      gameCoin: gameCoin,
      amountText: amountText,
    );
  }

  Future<bool> _submit({
    required bool deposit,
    required int gameAccountId,
    required String gameCoin,
    required String amountText,
  }) async {
    if (busy.value) return false;
    final amount = double.tryParse(amountText.trim());
    if (amount == null || amount <= 0) {
      toastification.show(
        title: const Text('Invalid amount'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 2),
      );
      return false;
    }
    final coin = gameCoin.trim();
    if (coin.isEmpty) {
      toastification.show(
        title: const Text('game_coin required'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 2),
      );
      return false;
    }
    busy.value = true;
    try {
      final txn = deposit
          ? await _wallet.deposit(
              gameAccountId: gameAccountId,
              gameCoin: coin,
              amount: amount,
            )
          : await _wallet.withdraw(
              gameAccountId: gameAccountId,
              gameCoin: coin,
              amount: amount,
            );
      toastification.show(
        title: Text(
          '${deposit ? 'Deposit' : 'Withdraw'} ${txn.status} · #${txn.id}',
        ),
        type: ToastificationType.success,
        autoCloseDuration: const Duration(seconds: 3),
      );
      await reload();
      return true;
    } catch (e) {
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
      return false;
    } finally {
      busy.value = false;
    }
  }
}
