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
  String get marketplace => 'Marketplace';

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

  @override
  String get marketplacePlaceDeal => 'Place deal';

  @override
  String get marketplaceTakeDeal => 'Take deal';

  @override
  String get marketplaceCancelDeal => 'Cancel deal';

  @override
  String get marketplaceEmpty => 'No open deals yet.';

  @override
  String get marketplaceEligible => 'You can place deals.';

  @override
  String get marketplaceNotEligible => 'Place deal blocked until gates pass.';

  @override
  String marketplaceListingFee(String fee) {
    return 'Listing fee: $fee PLT';
  }

  @override
  String get marketplaceListingFeeLabel => 'Listing fee';

  @override
  String get marketplaceGatePlt => 'Buy Platform Token';

  @override
  String get marketplaceGateGame => 'Bind a listed game';

  @override
  String get marketplaceGateTransfer => 'Partner Transfer API required';

  @override
  String get marketplaceWant => 'Want';

  @override
  String get marketplaceOffer => 'Offer';

  @override
  String get marketplaceGame => 'Game';

  @override
  String get marketplaceOfferType => 'Offer type';

  @override
  String get marketplaceOfferGameCoin => 'Game coin';

  @override
  String get marketplaceOfferCompanyCoin => 'Company coin';

  @override
  String get marketplaceOfferGameItem => 'Game item';

  @override
  String get marketplaceOfferCoinId => 'Offer game_coin_id';

  @override
  String get marketplaceOfferAmount => 'Offer amount';

  @override
  String get marketplaceWantType => 'Want type';

  @override
  String get marketplaceWantPlt => 'Platform Token';

  @override
  String get marketplaceWantCompanyCoin => 'Company coin';

  @override
  String get marketplaceWantCoinId => 'Want game_coin_id';

  @override
  String get marketplaceWantAmount => 'Want amount (price)';

  @override
  String get marketplaceItemRef => 'Item';

  @override
  String get marketplacePlaced => 'Deal posted';

  @override
  String get marketplaceTaken => 'Deal taken and settled';

  @override
  String get marketplaceTakeFailed => 'Take failed';

  @override
  String get marketplaceCancelled => 'Deal cancelled';

  @override
  String get marketplaceDealNotFound => 'Deal not found';

  @override
  String marketplaceDealTitle(int id) {
    return 'Deal #$id';
  }

  @override
  String get marketplaceYourGameAccount => 'Your game account';

  @override
  String get marketplaceBack => 'Back';

  @override
  String get corpRegisterTitle => 'Company registration';

  @override
  String get corpLoginTitle => 'Company login';

  @override
  String get corpHomeTitle => 'Company portal';

  @override
  String get corpRegisterHint =>
      'Create a company account and upload your Business Registration (BR). An admin must approve before you can submit games.';

  @override
  String get corpCompanyName => 'Company name';

  @override
  String get corpContactName => 'Contact name';

  @override
  String get corpContactEmail => 'Contact email';

  @override
  String get corpUploadBr => 'Upload Business Registration (BR)';

  @override
  String get corpBrSelected => 'BR file';

  @override
  String get corpBrRequired => 'BR document is required';

  @override
  String get corpSubmitRegistration => 'Submit for approval';

  @override
  String get corpRegisterSubmitted =>
      'Registration submitted — awaiting admin approval';

  @override
  String get corpHaveAccount => 'Already registered? Login';

  @override
  String get corpNoAccount => 'New company? Register';

  @override
  String get corpLoginField => 'Email or master code';

  @override
  String get corpNeedLogin => 'Please login';

  @override
  String get corpMasterCode => 'Master code';

  @override
  String corpSecretOnce(String secret) {
    return 'API secret (shown once): $secret';
  }

  @override
  String get corpPendingApproval =>
      'Your BR is under review. You can submit games after an admin approves your company.';

  @override
  String get corpRejected => 'Registration rejected';

  @override
  String get corpSubmitGame => 'Submit a game';

  @override
  String get corpGames => 'Your games';

  @override
  String get corpNoGames => 'No games submitted yet.';

  @override
  String get corpGameWizardHint =>
      'Submit game details plus government approval and/or approved materials. Status stays pending until admin review.';

  @override
  String get corpGameName => 'Game name';

  @override
  String get corpGameCode => 'Game code';

  @override
  String get corpPartnerCode => 'Partner code (optional)';

  @override
  String get corpPartnerGameId => 'Partner game id (optional)';

  @override
  String get corpGameDescription => 'Description';

  @override
  String get corpUploadGovDoc => 'Upload government approval document';

  @override
  String get corpUploadMaterial => 'Upload approved materials';

  @override
  String get corpGovSelected => 'Government doc';

  @override
  String get corpMaterialSelected => 'Materials';

  @override
  String get corpGameDocRequired => 'Upload at least one document';

  @override
  String get corpGameSubmitted => 'Game submitted for review';
}
