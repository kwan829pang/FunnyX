import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('zh'),
    Locale('zh', 'CN'),
    Locale('zh', 'TW'),
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'FunnyX'**
  String get appTitle;

  /// No description provided for @login.
  ///
  /// In en, this message translates to:
  /// **'Login'**
  String get login;

  /// No description provided for @register.
  ///
  /// In en, this message translates to:
  /// **'Register'**
  String get register;

  /// No description provided for @username.
  ///
  /// In en, this message translates to:
  /// **'Username'**
  String get username;

  /// No description provided for @password.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get password;

  /// No description provided for @home.
  ///
  /// In en, this message translates to:
  /// **'Home'**
  String get home;

  /// No description provided for @wallet.
  ///
  /// In en, this message translates to:
  /// **'Wallet'**
  String get wallet;

  /// No description provided for @walletProfile.
  ///
  /// In en, this message translates to:
  /// **'Profile'**
  String get walletProfile;

  /// No description provided for @walletEmail.
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get walletEmail;

  /// No description provided for @walletHint.
  ///
  /// In en, this message translates to:
  /// **'Balances update after shop settle or deposit/withdraw partner callbacks. Refresh to reload.'**
  String get walletHint;

  /// No description provided for @walletBalances.
  ///
  /// In en, this message translates to:
  /// **'Balances'**
  String get walletBalances;

  /// No description provided for @walletTransactions.
  ///
  /// In en, this message translates to:
  /// **'Transactions'**
  String get walletTransactions;

  /// No description provided for @walletEmptyBalances.
  ///
  /// In en, this message translates to:
  /// **'No wallet balances yet.'**
  String get walletEmptyBalances;

  /// No description provided for @walletEmptyTxns.
  ///
  /// In en, this message translates to:
  /// **'No money transactions yet.'**
  String get walletEmptyTxns;

  /// No description provided for @walletAvailable.
  ///
  /// In en, this message translates to:
  /// **'Available'**
  String get walletAvailable;

  /// No description provided for @walletLocked.
  ///
  /// In en, this message translates to:
  /// **'Locked'**
  String get walletLocked;

  /// No description provided for @walletGameCoin.
  ///
  /// In en, this message translates to:
  /// **'Game coin'**
  String get walletGameCoin;

  /// No description provided for @walletAmount.
  ///
  /// In en, this message translates to:
  /// **'Amount'**
  String get walletAmount;

  /// No description provided for @walletDeposit.
  ///
  /// In en, this message translates to:
  /// **'Deposit'**
  String get walletDeposit;

  /// No description provided for @walletWithdraw.
  ///
  /// In en, this message translates to:
  /// **'Withdraw'**
  String get walletWithdraw;

  /// No description provided for @walletTxnId.
  ///
  /// In en, this message translates to:
  /// **'Txn'**
  String get walletTxnId;

  /// No description provided for @walletTxnType.
  ///
  /// In en, this message translates to:
  /// **'Type'**
  String get walletTxnType;

  /// No description provided for @walletPltAvailable.
  ///
  /// In en, this message translates to:
  /// **'PLT available'**
  String get walletPltAvailable;

  /// No description provided for @walletBalanceCount.
  ///
  /// In en, this message translates to:
  /// **'{count} balance rows'**
  String walletBalanceCount(int count);

  /// No description provided for @shop.
  ///
  /// In en, this message translates to:
  /// **'E-shop'**
  String get shop;

  /// No description provided for @marketplace.
  ///
  /// In en, this message translates to:
  /// **'Marketplace'**
  String get marketplace;

  /// No description provided for @logout.
  ///
  /// In en, this message translates to:
  /// **'Logout'**
  String get logout;

  /// No description provided for @loginHint.
  ///
  /// In en, this message translates to:
  /// **'Password login, register, or Partner OAuth. Session APIs need a Token Server token.'**
  String get loginHint;

  /// No description provided for @registerOauthHint.
  ///
  /// In en, this message translates to:
  /// **'Register on FunnyX (Path A). Partners can then use Platform OAuth so you login in their game. Or continue with Partner OAuth if you already have a partner account (Path B).'**
  String get registerOauthHint;

  /// No description provided for @noAccount.
  ///
  /// In en, this message translates to:
  /// **'No account? Register'**
  String get noAccount;

  /// No description provided for @haveAccount.
  ///
  /// In en, this message translates to:
  /// **'Have an account? Login'**
  String get haveAccount;

  /// No description provided for @oauthCompareTitle.
  ///
  /// In en, this message translates to:
  /// **'OAuth 2.0 — two directions'**
  String get oauthCompareTitle;

  /// No description provided for @oauthPathBTitle.
  ///
  /// In en, this message translates to:
  /// **'Path B — Partner → Platform'**
  String get oauthPathBTitle;

  /// No description provided for @oauthPathBBody.
  ///
  /// In en, this message translates to:
  /// **'You registered on the Partner game/site. Client Web starts Partner OAuth; Session Token Server brokers code/userinfo and issues your FunnyX session (sts_…).'**
  String get oauthPathBBody;

  /// No description provided for @oauthPathCTitle.
  ///
  /// In en, this message translates to:
  /// **'Path C — Platform → Partner'**
  String get oauthPathCTitle;

  /// No description provided for @oauthPathCBody.
  ///
  /// In en, this message translates to:
  /// **'You registered on Client Web first. The Partner redirects to FunnyX Platform OAuth (/v1/oauth/authorize). After you consent, the Partner receives a code and logs you into their game.'**
  String get oauthPathCBody;

  /// No description provided for @oauthContinuePartner.
  ///
  /// In en, this message translates to:
  /// **'Continue with Partner OAuth'**
  String get oauthContinuePartner;

  /// No description provided for @oauthPlatformAuthorizeHint.
  ///
  /// In en, this message translates to:
  /// **'Authorize URL Partners embed (Path C demo):'**
  String get oauthPlatformAuthorizeHint;

  /// No description provided for @oauthCopyAuthorize.
  ///
  /// In en, this message translates to:
  /// **'Copy authorize URL'**
  String get oauthCopyAuthorize;

  /// No description provided for @oauthCopied.
  ///
  /// In en, this message translates to:
  /// **'Copied'**
  String get oauthCopied;

  /// No description provided for @oauthCompleting.
  ///
  /// In en, this message translates to:
  /// **'Completing Partner OAuth…'**
  String get oauthCompleting;

  /// No description provided for @oauthSuccess.
  ///
  /// In en, this message translates to:
  /// **'Partner OAuth login complete'**
  String get oauthSuccess;

  /// No description provided for @oauthFailed.
  ///
  /// In en, this message translates to:
  /// **'Partner OAuth failed'**
  String get oauthFailed;

  /// No description provided for @linkPartnerAccount.
  ///
  /// In en, this message translates to:
  /// **'Link Partner account (OAuth)'**
  String get linkPartnerAccount;

  /// No description provided for @shopCart.
  ///
  /// In en, this message translates to:
  /// **'Cart'**
  String get shopCart;

  /// No description provided for @shopOrders.
  ///
  /// In en, this message translates to:
  /// **'Orders'**
  String get shopOrders;

  /// No description provided for @shopCheckout.
  ///
  /// In en, this message translates to:
  /// **'Checkout'**
  String get shopCheckout;

  /// No description provided for @shopPayment.
  ///
  /// In en, this message translates to:
  /// **'Payment'**
  String get shopPayment;

  /// No description provided for @shopRetry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get shopRetry;

  /// No description provided for @shopEmptyCatalog.
  ///
  /// In en, this message translates to:
  /// **'No products available.'**
  String get shopEmptyCatalog;

  /// No description provided for @shopBaseCurrency.
  ///
  /// In en, this message translates to:
  /// **'Prices in {currency}'**
  String shopBaseCurrency(String currency);

  /// No description provided for @shopSellerPlatform.
  ///
  /// In en, this message translates to:
  /// **'Platform'**
  String get shopSellerPlatform;

  /// No description provided for @shopSellerCorp.
  ///
  /// In en, this message translates to:
  /// **'Corp'**
  String get shopSellerCorp;

  /// No description provided for @shopAddedToCart.
  ///
  /// In en, this message translates to:
  /// **'Added to cart'**
  String get shopAddedToCart;

  /// No description provided for @shopCartEmpty.
  ///
  /// In en, this message translates to:
  /// **'Your cart is empty.'**
  String get shopCartEmpty;

  /// No description provided for @shopColItem.
  ///
  /// In en, this message translates to:
  /// **'Item'**
  String get shopColItem;

  /// No description provided for @shopColUnitPrice.
  ///
  /// In en, this message translates to:
  /// **'Unit price'**
  String get shopColUnitPrice;

  /// No description provided for @shopColQty.
  ///
  /// In en, this message translates to:
  /// **'Qty'**
  String get shopColQty;

  /// No description provided for @shopColLineTotal.
  ///
  /// In en, this message translates to:
  /// **'Line total'**
  String get shopColLineTotal;

  /// No description provided for @shopColActions.
  ///
  /// In en, this message translates to:
  /// **'Actions'**
  String get shopColActions;

  /// No description provided for @shopColOrderId.
  ///
  /// In en, this message translates to:
  /// **'Order'**
  String get shopColOrderId;

  /// No description provided for @shopColSeller.
  ///
  /// In en, this message translates to:
  /// **'Seller'**
  String get shopColSeller;

  /// No description provided for @shopColFiat.
  ///
  /// In en, this message translates to:
  /// **'Fiat'**
  String get shopColFiat;

  /// No description provided for @shopColStatus.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get shopColStatus;

  /// No description provided for @shopColExpires.
  ///
  /// In en, this message translates to:
  /// **'Expires'**
  String get shopColExpires;

  /// No description provided for @shopSubtotal.
  ///
  /// In en, this message translates to:
  /// **'Subtotal: {amount}'**
  String shopSubtotal(String amount);

  /// No description provided for @shopSelectGameAccount.
  ///
  /// In en, this message translates to:
  /// **'Select game account for fulfillment'**
  String get shopSelectGameAccount;

  /// No description provided for @shopGameAccount.
  ///
  /// In en, this message translates to:
  /// **'Game account'**
  String get shopGameAccount;

  /// No description provided for @shopNoGameAccount.
  ///
  /// In en, this message translates to:
  /// **'No active game account binding. Bind a game account first (home or partner OAuth).'**
  String get shopNoGameAccount;

  /// No description provided for @shopOrderSummary.
  ///
  /// In en, this message translates to:
  /// **'Order summary'**
  String get shopOrderSummary;

  /// No description provided for @shopPlaceOrder.
  ///
  /// In en, this message translates to:
  /// **'Place order'**
  String get shopPlaceOrder;

  /// No description provided for @shopPaymentHint.
  ///
  /// In en, this message translates to:
  /// **'Pending orders stay payable for 24 hours. Pay via partner checkout, then refresh status.'**
  String get shopPaymentHint;

  /// No description provided for @shopNoPendingOrders.
  ///
  /// In en, this message translates to:
  /// **'No pending payment orders.'**
  String get shopNoPendingOrders;

  /// No description provided for @shopNoOrders.
  ///
  /// In en, this message translates to:
  /// **'No orders yet.'**
  String get shopNoOrders;

  /// No description provided for @shopPay.
  ///
  /// In en, this message translates to:
  /// **'Pay'**
  String get shopPay;

  /// No description provided for @shopCancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get shopCancel;

  /// No description provided for @shopCancelled.
  ///
  /// In en, this message translates to:
  /// **'Order cancelled'**
  String get shopCancelled;

  /// No description provided for @shopOrderDetail.
  ///
  /// In en, this message translates to:
  /// **'Order #{id}'**
  String shopOrderDetail(int id);

  /// No description provided for @shopOrderNotFound.
  ///
  /// In en, this message translates to:
  /// **'Order not found'**
  String get shopOrderNotFound;

  /// No description provided for @shopCredit.
  ///
  /// In en, this message translates to:
  /// **'Credit'**
  String get shopCredit;

  /// No description provided for @shopCreatedAt.
  ///
  /// In en, this message translates to:
  /// **'Created'**
  String get shopCreatedAt;

  /// No description provided for @shopPartnerOrder.
  ///
  /// In en, this message translates to:
  /// **'Partner order'**
  String get shopPartnerOrder;

  /// No description provided for @marketplacePlaceDeal.
  ///
  /// In en, this message translates to:
  /// **'Place deal'**
  String get marketplacePlaceDeal;

  /// No description provided for @marketplaceTakeDeal.
  ///
  /// In en, this message translates to:
  /// **'Take deal'**
  String get marketplaceTakeDeal;

  /// No description provided for @marketplaceCancelDeal.
  ///
  /// In en, this message translates to:
  /// **'Cancel deal'**
  String get marketplaceCancelDeal;

  /// No description provided for @marketplaceEmpty.
  ///
  /// In en, this message translates to:
  /// **'No open deals yet.'**
  String get marketplaceEmpty;

  /// No description provided for @marketplaceEligible.
  ///
  /// In en, this message translates to:
  /// **'You can place deals.'**
  String get marketplaceEligible;

  /// No description provided for @marketplaceNotEligible.
  ///
  /// In en, this message translates to:
  /// **'Place deal blocked until gates pass.'**
  String get marketplaceNotEligible;

  /// No description provided for @marketplaceListingFee.
  ///
  /// In en, this message translates to:
  /// **'Listing fee: {fee} PLT'**
  String marketplaceListingFee(String fee);

  /// No description provided for @marketplaceListingFeeLabel.
  ///
  /// In en, this message translates to:
  /// **'Listing fee'**
  String get marketplaceListingFeeLabel;

  /// No description provided for @marketplaceGatePlt.
  ///
  /// In en, this message translates to:
  /// **'Buy Platform Token'**
  String get marketplaceGatePlt;

  /// No description provided for @marketplaceGateGame.
  ///
  /// In en, this message translates to:
  /// **'Bind a listed game'**
  String get marketplaceGateGame;

  /// No description provided for @marketplaceGateTransfer.
  ///
  /// In en, this message translates to:
  /// **'Partner Transfer API required'**
  String get marketplaceGateTransfer;

  /// No description provided for @marketplaceWant.
  ///
  /// In en, this message translates to:
  /// **'Want'**
  String get marketplaceWant;

  /// No description provided for @marketplaceOffer.
  ///
  /// In en, this message translates to:
  /// **'Offer'**
  String get marketplaceOffer;

  /// No description provided for @marketplaceGame.
  ///
  /// In en, this message translates to:
  /// **'Game'**
  String get marketplaceGame;

  /// No description provided for @marketplaceOfferType.
  ///
  /// In en, this message translates to:
  /// **'Offer type'**
  String get marketplaceOfferType;

  /// No description provided for @marketplaceOfferGameCoin.
  ///
  /// In en, this message translates to:
  /// **'Game coin'**
  String get marketplaceOfferGameCoin;

  /// No description provided for @marketplaceOfferCompanyCoin.
  ///
  /// In en, this message translates to:
  /// **'Company coin'**
  String get marketplaceOfferCompanyCoin;

  /// No description provided for @marketplaceOfferGameItem.
  ///
  /// In en, this message translates to:
  /// **'Game item'**
  String get marketplaceOfferGameItem;

  /// No description provided for @marketplaceOfferCoinId.
  ///
  /// In en, this message translates to:
  /// **'Offer game_coin_id'**
  String get marketplaceOfferCoinId;

  /// No description provided for @marketplaceOfferAmount.
  ///
  /// In en, this message translates to:
  /// **'Offer amount'**
  String get marketplaceOfferAmount;

  /// No description provided for @marketplaceWantType.
  ///
  /// In en, this message translates to:
  /// **'Want type'**
  String get marketplaceWantType;

  /// No description provided for @marketplaceWantPlt.
  ///
  /// In en, this message translates to:
  /// **'Platform Token'**
  String get marketplaceWantPlt;

  /// No description provided for @marketplaceWantCompanyCoin.
  ///
  /// In en, this message translates to:
  /// **'Company coin'**
  String get marketplaceWantCompanyCoin;

  /// No description provided for @marketplaceWantCoinId.
  ///
  /// In en, this message translates to:
  /// **'Want game_coin_id'**
  String get marketplaceWantCoinId;

  /// No description provided for @marketplaceWantAmount.
  ///
  /// In en, this message translates to:
  /// **'Want amount (price)'**
  String get marketplaceWantAmount;

  /// No description provided for @marketplaceItemRef.
  ///
  /// In en, this message translates to:
  /// **'Item'**
  String get marketplaceItemRef;

  /// No description provided for @marketplacePlaced.
  ///
  /// In en, this message translates to:
  /// **'Deal posted'**
  String get marketplacePlaced;

  /// No description provided for @marketplaceTaken.
  ///
  /// In en, this message translates to:
  /// **'Deal taken and settled'**
  String get marketplaceTaken;

  /// No description provided for @marketplaceTakeFailed.
  ///
  /// In en, this message translates to:
  /// **'Take failed'**
  String get marketplaceTakeFailed;

  /// No description provided for @marketplaceCancelled.
  ///
  /// In en, this message translates to:
  /// **'Deal cancelled'**
  String get marketplaceCancelled;

  /// No description provided for @marketplaceDealNotFound.
  ///
  /// In en, this message translates to:
  /// **'Deal not found'**
  String get marketplaceDealNotFound;

  /// No description provided for @marketplaceDealTitle.
  ///
  /// In en, this message translates to:
  /// **'Deal #{id}'**
  String marketplaceDealTitle(int id);

  /// No description provided for @marketplaceYourGameAccount.
  ///
  /// In en, this message translates to:
  /// **'Your game account'**
  String get marketplaceYourGameAccount;

  /// No description provided for @marketplaceBack.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get marketplaceBack;

  /// No description provided for @corpRegisterTitle.
  ///
  /// In en, this message translates to:
  /// **'Company registration'**
  String get corpRegisterTitle;

  /// No description provided for @corpLoginTitle.
  ///
  /// In en, this message translates to:
  /// **'Company login'**
  String get corpLoginTitle;

  /// No description provided for @corpHomeTitle.
  ///
  /// In en, this message translates to:
  /// **'Company portal'**
  String get corpHomeTitle;

  /// No description provided for @corpRegisterHint.
  ///
  /// In en, this message translates to:
  /// **'Create a company account and upload your Business Registration (BR). An admin must approve before you can submit games.'**
  String get corpRegisterHint;

  /// No description provided for @corpCompanyName.
  ///
  /// In en, this message translates to:
  /// **'Company name'**
  String get corpCompanyName;

  /// No description provided for @corpContactName.
  ///
  /// In en, this message translates to:
  /// **'Contact name'**
  String get corpContactName;

  /// No description provided for @corpContactEmail.
  ///
  /// In en, this message translates to:
  /// **'Contact email'**
  String get corpContactEmail;

  /// No description provided for @corpUploadBr.
  ///
  /// In en, this message translates to:
  /// **'Upload Business Registration (BR)'**
  String get corpUploadBr;

  /// No description provided for @corpBrSelected.
  ///
  /// In en, this message translates to:
  /// **'BR file'**
  String get corpBrSelected;

  /// No description provided for @corpBrRequired.
  ///
  /// In en, this message translates to:
  /// **'BR document is required'**
  String get corpBrRequired;

  /// No description provided for @corpSubmitRegistration.
  ///
  /// In en, this message translates to:
  /// **'Submit for approval'**
  String get corpSubmitRegistration;

  /// No description provided for @corpRegisterSubmitted.
  ///
  /// In en, this message translates to:
  /// **'Registration submitted — awaiting admin approval'**
  String get corpRegisterSubmitted;

  /// No description provided for @corpHaveAccount.
  ///
  /// In en, this message translates to:
  /// **'Already registered? Login'**
  String get corpHaveAccount;

  /// No description provided for @corpNoAccount.
  ///
  /// In en, this message translates to:
  /// **'New company? Register'**
  String get corpNoAccount;

  /// No description provided for @corpLoginField.
  ///
  /// In en, this message translates to:
  /// **'Email or master code'**
  String get corpLoginField;

  /// No description provided for @corpNeedLogin.
  ///
  /// In en, this message translates to:
  /// **'Please login'**
  String get corpNeedLogin;

  /// No description provided for @corpMasterCode.
  ///
  /// In en, this message translates to:
  /// **'Master code'**
  String get corpMasterCode;

  /// No description provided for @corpSecretOnce.
  ///
  /// In en, this message translates to:
  /// **'API secret (shown once): {secret}'**
  String corpSecretOnce(String secret);

  /// No description provided for @corpPendingApproval.
  ///
  /// In en, this message translates to:
  /// **'Your BR is under review. You can submit games after an admin approves your company.'**
  String get corpPendingApproval;

  /// No description provided for @corpRejected.
  ///
  /// In en, this message translates to:
  /// **'Registration rejected'**
  String get corpRejected;

  /// No description provided for @corpSubmitGame.
  ///
  /// In en, this message translates to:
  /// **'Submit a game'**
  String get corpSubmitGame;

  /// No description provided for @corpGames.
  ///
  /// In en, this message translates to:
  /// **'Your games'**
  String get corpGames;

  /// No description provided for @corpNoGames.
  ///
  /// In en, this message translates to:
  /// **'No games submitted yet.'**
  String get corpNoGames;

  /// No description provided for @corpGameWizardHint.
  ///
  /// In en, this message translates to:
  /// **'Submit game details plus government approval and/or approved materials. Status stays pending until admin review.'**
  String get corpGameWizardHint;

  /// No description provided for @corpGameName.
  ///
  /// In en, this message translates to:
  /// **'Game name'**
  String get corpGameName;

  /// No description provided for @corpGameCode.
  ///
  /// In en, this message translates to:
  /// **'Game code'**
  String get corpGameCode;

  /// No description provided for @corpPartnerCode.
  ///
  /// In en, this message translates to:
  /// **'Partner code (optional)'**
  String get corpPartnerCode;

  /// No description provided for @corpPartnerGameId.
  ///
  /// In en, this message translates to:
  /// **'Partner game id (optional)'**
  String get corpPartnerGameId;

  /// No description provided for @corpGameDescription.
  ///
  /// In en, this message translates to:
  /// **'Description'**
  String get corpGameDescription;

  /// No description provided for @corpUploadGovDoc.
  ///
  /// In en, this message translates to:
  /// **'Upload government approval document'**
  String get corpUploadGovDoc;

  /// No description provided for @corpUploadMaterial.
  ///
  /// In en, this message translates to:
  /// **'Upload approved materials'**
  String get corpUploadMaterial;

  /// No description provided for @corpGovSelected.
  ///
  /// In en, this message translates to:
  /// **'Government doc'**
  String get corpGovSelected;

  /// No description provided for @corpMaterialSelected.
  ///
  /// In en, this message translates to:
  /// **'Materials'**
  String get corpMaterialSelected;

  /// No description provided for @corpGameDocRequired.
  ///
  /// In en, this message translates to:
  /// **'Upload at least one document'**
  String get corpGameDocRequired;

  /// No description provided for @corpGameSubmitted.
  ///
  /// In en, this message translates to:
  /// **'Game submitted for review'**
  String get corpGameSubmitted;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when language+country codes are specified.
  switch (locale.languageCode) {
    case 'zh':
      {
        switch (locale.countryCode) {
          case 'CN':
            return AppLocalizationsZhCn();
          case 'TW':
            return AppLocalizationsZhTw();
        }
        break;
      }
  }

  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
