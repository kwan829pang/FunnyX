// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'FunnyX';

  @override
  String get login => 'Login';

  @override
  String get register => 'Register';

  @override
  String get username => 'Username';

  @override
  String get password => 'Password';

  @override
  String get home => 'Home';

  @override
  String get shop => 'E-shop';

  @override
  String get logout => 'Logout';

  @override
  String get loginHint =>
      'Password login, register, or Partner OAuth. Session APIs need a Token Server token.';

  @override
  String get registerOauthHint =>
      'Register on FunnyX (Path A). Partners can then use Platform OAuth so you login in their game. Or continue with Partner OAuth if you already have a partner account (Path B).';

  @override
  String get noAccount => 'No account? Register';

  @override
  String get haveAccount => 'Have an account? Login';

  @override
  String get oauthCompareTitle => 'OAuth 2.0 — two directions';

  @override
  String get oauthPathBTitle => 'Path B — Partner → Platform';

  @override
  String get oauthPathBBody =>
      'You registered on the Partner game/site. Client Web starts Partner OAuth; Session Token Server brokers code/userinfo and issues your FunnyX session (sts_…).';

  @override
  String get oauthPathCTitle => 'Path C — Platform → Partner';

  @override
  String get oauthPathCBody =>
      'You registered on Client Web first. The Partner redirects to FunnyX Platform OAuth (/v1/oauth/authorize). After you consent, the Partner receives a code and logs you into their game.';

  @override
  String get oauthContinuePartner => 'Continue with Partner OAuth';

  @override
  String get oauthPlatformAuthorizeHint =>
      'Authorize URL Partners embed (Path C demo):';

  @override
  String get oauthCopyAuthorize => 'Copy authorize URL';

  @override
  String get oauthCopied => 'Copied';

  @override
  String get oauthCompleting => 'Completing Partner OAuth…';

  @override
  String get oauthSuccess => 'Partner OAuth login complete';

  @override
  String get oauthFailed => 'Partner OAuth failed';

  @override
  String get linkPartnerAccount => 'Link Partner account (OAuth)';

  @override
  String get shopCart => 'Cart';

  @override
  String get shopOrders => 'Orders';

  @override
  String get shopCheckout => 'Checkout';

  @override
  String get shopPayment => 'Payment';

  @override
  String get shopRetry => 'Retry';

  @override
  String get shopEmptyCatalog => 'No products available.';

  @override
  String shopBaseCurrency(String currency) {
    return 'Prices in $currency';
  }

  @override
  String get shopSellerPlatform => 'Platform';

  @override
  String get shopSellerCorp => 'Corp';

  @override
  String get shopAddedToCart => 'Added to cart';

  @override
  String get shopCartEmpty => 'Your cart is empty.';

  @override
  String get shopColItem => 'Item';

  @override
  String get shopColUnitPrice => 'Unit price';

  @override
  String get shopColQty => 'Qty';

  @override
  String get shopColLineTotal => 'Line total';

  @override
  String get shopColActions => 'Actions';

  @override
  String get shopColOrderId => 'Order';

  @override
  String get shopColSeller => 'Seller';

  @override
  String get shopColFiat => 'Fiat';

  @override
  String get shopColStatus => 'Status';

  @override
  String get shopColExpires => 'Expires';

  @override
  String shopSubtotal(String amount) {
    return 'Subtotal: $amount';
  }

  @override
  String get shopSelectGameAccount => 'Select game account for fulfillment';

  @override
  String get shopGameAccount => 'Game account';

  @override
  String get shopNoGameAccount =>
      'No active game account binding. Bind a game account first (home or partner OAuth).';

  @override
  String get shopOrderSummary => 'Order summary';

  @override
  String get shopPlaceOrder => 'Place order';

  @override
  String get shopPaymentHint =>
      'Pending orders stay payable for 24 hours. Pay via partner checkout, then refresh status.';

  @override
  String get shopNoPendingOrders => 'No pending payment orders.';

  @override
  String get shopNoOrders => 'No orders yet.';

  @override
  String get shopPay => 'Pay';

  @override
  String get shopCancel => 'Cancel';

  @override
  String get shopCancelled => 'Order cancelled';

  @override
  String shopOrderDetail(int id) {
    return 'Order #$id';
  }

  @override
  String get shopOrderNotFound => 'Order not found';

  @override
  String get shopCredit => 'Credit';

  @override
  String get shopCreatedAt => 'Created';

  @override
  String get shopPartnerOrder => 'Partner order';
}
