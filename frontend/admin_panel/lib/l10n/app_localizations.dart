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
  /// **'FunnyX Admin'**
  String get appTitle;

  /// No description provided for @login.
  ///
  /// In en, this message translates to:
  /// **'Login'**
  String get login;

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

  /// No description provided for @dashboard.
  ///
  /// In en, this message translates to:
  /// **'Dashboard'**
  String get dashboard;

  /// No description provided for @markets.
  ///
  /// In en, this message translates to:
  /// **'Markets'**
  String get markets;

  /// No description provided for @packages.
  ///
  /// In en, this message translates to:
  /// **'PLT packages'**
  String get packages;

  /// No description provided for @corpProducts.
  ///
  /// In en, this message translates to:
  /// **'Corp products'**
  String get corpProducts;

  /// No description provided for @setup.
  ///
  /// In en, this message translates to:
  /// **'System setup'**
  String get setup;

  /// No description provided for @logout.
  ///
  /// In en, this message translates to:
  /// **'Logout'**
  String get logout;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @status.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get status;

  /// No description provided for @actions.
  ///
  /// In en, this message translates to:
  /// **'Actions'**
  String get actions;

  /// No description provided for @commandPalette.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get commandPalette;

  /// No description provided for @loginHint.
  ///
  /// In en, this message translates to:
  /// **'Public login. Other pages require a session token.'**
  String get loginHint;

  /// No description provided for @setupWizardTitle.
  ///
  /// In en, this message translates to:
  /// **'System setup wizard'**
  String get setupWizardTitle;

  /// No description provided for @setupWizardSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Configure required platform variables before using Admin Panel.'**
  String get setupWizardSubtitle;

  /// No description provided for @setupStepWelcome.
  ///
  /// In en, this message translates to:
  /// **'Required variables'**
  String get setupStepWelcome;

  /// No description provided for @setupStepBaseFiat.
  ///
  /// In en, this message translates to:
  /// **'Base fiat currency'**
  String get setupStepBaseFiat;

  /// No description provided for @setupStepPackages.
  ///
  /// In en, this message translates to:
  /// **'PLT package prices'**
  String get setupStepPackages;

  /// No description provided for @setupStepConfirm.
  ///
  /// In en, this message translates to:
  /// **'Confirm'**
  String get setupStepConfirm;

  /// No description provided for @setupWelcomeBody.
  ///
  /// In en, this message translates to:
  /// **'FunnyX needs a base fiat currency (HKD or USD) and at least one priced platform PLT package for the e-shop and Corp fees.'**
  String get setupWelcomeBody;

  /// No description provided for @setupBaseFiatHint.
  ///
  /// In en, this message translates to:
  /// **'This currency applies to all e-shop list prices and C6 Corp fee invoices.'**
  String get setupBaseFiatHint;

  /// No description provided for @setupPackagesHint.
  ///
  /// In en, this message translates to:
  /// **'Set fiat prices for fixed Platform Token packages (PLT_*). At least one must stay active.'**
  String get setupPackagesHint;

  /// No description provided for @setupConfirmHint.
  ///
  /// In en, this message translates to:
  /// **'Review the values below, then finish to mark the system as initialized.'**
  String get setupConfirmHint;

  /// No description provided for @setupFiatPrice.
  ///
  /// In en, this message translates to:
  /// **'Fiat price'**
  String get setupFiatPrice;

  /// No description provided for @setupNext.
  ///
  /// In en, this message translates to:
  /// **'Next'**
  String get setupNext;

  /// No description provided for @setupSaveContinue.
  ///
  /// In en, this message translates to:
  /// **'Save and continue'**
  String get setupSaveContinue;

  /// No description provided for @setupFinish.
  ///
  /// In en, this message translates to:
  /// **'Finish setup'**
  String get setupFinish;

  /// No description provided for @packagesHint.
  ///
  /// In en, this message translates to:
  /// **'Seed and manage fixed Platform Token packages. Codes must start with PLT_.'**
  String get packagesHint;

  /// No description provided for @packagesEmpty.
  ///
  /// In en, this message translates to:
  /// **'No platform packages yet.'**
  String get packagesEmpty;

  /// No description provided for @packagesCreate.
  ///
  /// In en, this message translates to:
  /// **'Create package'**
  String get packagesCreate;

  /// No description provided for @packagesCode.
  ///
  /// In en, this message translates to:
  /// **'Code'**
  String get packagesCode;

  /// No description provided for @packagesName.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get packagesName;

  /// No description provided for @packagesCoinAmount.
  ///
  /// In en, this message translates to:
  /// **'Coin amount'**
  String get packagesCoinAmount;

  /// No description provided for @corpProductsHint.
  ///
  /// In en, this message translates to:
  /// **'Monitor Corp e-shop products. Suspend with inactive or archive.'**
  String get corpProductsHint;

  /// No description provided for @corpProductsEmpty.
  ///
  /// In en, this message translates to:
  /// **'No Corp products found.'**
  String get corpProductsEmpty;

  /// No description provided for @corpId.
  ///
  /// In en, this message translates to:
  /// **'Corp ID'**
  String get corpId;

  /// No description provided for @productType.
  ///
  /// In en, this message translates to:
  /// **'Type'**
  String get productType;

  /// No description provided for @statusFilterAll.
  ///
  /// In en, this message translates to:
  /// **'All statuses'**
  String get statusFilterAll;

  /// No description provided for @statusDraft.
  ///
  /// In en, this message translates to:
  /// **'Draft'**
  String get statusDraft;

  /// No description provided for @statusActive.
  ///
  /// In en, this message translates to:
  /// **'Active'**
  String get statusActive;

  /// No description provided for @statusInactive.
  ///
  /// In en, this message translates to:
  /// **'Inactive'**
  String get statusInactive;

  /// No description provided for @statusArchived.
  ///
  /// In en, this message translates to:
  /// **'Archived'**
  String get statusArchived;
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
