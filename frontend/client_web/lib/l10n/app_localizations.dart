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

  /// No description provided for @shop.
  ///
  /// In en, this message translates to:
  /// **'E-shop'**
  String get shop;

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
