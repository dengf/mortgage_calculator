// Source-of-truth catalog. Every other locale mirrors these keys; a key
// missing elsewhere falls back to the string here rather than rendering
// blank.
//
// Keys are namespaced by where they appear (`nav.`, `payment.`, `sg.`) so
// a string's home is obvious when one needs changing. `err.` and `warn.`
// mirror codes emitted by the Rust layer — see crates/mortgage-core's
// MortgageError and crates/mortgage-wasm/src/singapore.rs.

export default {
  // App shell
  'app.title': 'Mortgage Calculator',
  'meta.title': 'Mortgage Calculator — Payments, Amortization, Affordability & Refinance',
  'meta.ogTitle': 'Mortgage Calculator — nothing leaves your device',
  'meta.description':
    'Free mortgage calculator for payments, amortization, affordability and refinancing, with Singapore TDSR, CPF and stamp duty built in. Runs entirely in your browser — your numbers never leave your device.',
  'intro.lead':
    'Work out what a home really costs — payments, amortization, what you can afford, and whether refinancing pays off.',
  'intro.privacy':
    'Everything is calculated in your browser. Nothing you type is sent anywhere, stored, or logged.',
  'intro.verify':
    "Don't take our word for it — open your browser's network tab and watch it stay empty.",
  'intro.depth.US':
    'Property tax estimated from your ZIP code, PMI, and 2026 conforming loan limits.',
  'intro.depth.SG': 'MAS TDSR and MSR limits, CPF, LTV ceilings, and IRAS stamp duty.',
  'about.title': 'How these numbers are worked out',

  'about.us.payment.q': 'What does the monthly payment include?',
  'about.us.payment.a':
    'The headline figure is principal and interest only. The US panel below adds property tax, estimated from your ZIP code, and PMI where the deposit is under 20% — together making the full PITI figure.',
  'about.us.pmi.q': 'When does PMI stop?',
  'about.us.pmi.a':
    'Private mortgage insurance applies while the deposit is under 20% of the price. You can request cancellation once you hold 20% equity, and lenders must drop it automatically at 78% loan-to-value under the Homeowners Protection Act.',
  'about.us.jumbo.q': 'What makes a loan jumbo?',
  'about.us.jumbo.a':
    'A loan above the FHFA conforming limit — $832,750 for a one-unit property in 2026 — cannot be bought by Fannie Mae or Freddie Mac, so it is priced as a jumbo. High-cost counties, Alaska and Hawaii have higher limits, which this calculator does not yet apply.',

  'about.sg.payment.q': 'What does the monthly payment include?',
  'about.sg.payment.a':
    'The headline figure is principal and interest only. The Singapore panel below splits it between CPF Ordinary Account and cash, and prices the stamp duty and deposit you need at completion.',
  'about.sg.tdsr.q': 'What are TDSR and MSR?',
  'about.sg.tdsr.a':
    'Singapore caps how much of your income can service debt. TDSR limits all debt repayments to 55% of gross monthly income; MSR limits the housing loan alone to 30%, and applies only to HDB flats and Executive Condominiums. Banks assess both at the higher of 4% or the rate your loan runs at after the lock-in — not the promotional rate you are quoted — so the ratios here use that assessed figure rather than your quoted payment.',
  'about.sg.afford.q': 'Why is my affordability lower than I expected?',
  'about.sg.afford.a':
    'Three rules usually bite before income does: the LTV ceiling caps a first housing loan at 75% of price, the minimum cash portion of the deposit cannot come from CPF, and both stamp duties fall due in cash within 14 days. Commission and bonus also count at only 70%.',

  'about.refi.q': 'When does refinancing actually pay off?',
  'about.refi.a':
    'When you stay in the home past the break-even point — the month at which cumulative savings overtake the closing costs. Watch the term as well as the rate: refinancing into a fresh 30-year loan lowers the payment but can raise the total paid.',

  'about.disclaimer.US':
    'Estimates for planning, not financial advice or a loan offer. Property tax rates are state averages and vary by county; loan limits and rules change.',
  'about.disclaimer.SG':
    'Estimates for planning, not financial advice or a loan offer. MAS and IRAS rules reflect published figures and can change; confirm stamp duty and any remission with IRAS before you commit.',
  'app.footer':
    'Calculations run entirely client-side, compiled from Rust to WebAssembly. Your numbers never leave your device.',
  'app.privacy': 'Privacy',
  'app.source': 'Source',
  'app.loading': 'Loading calculator...',
  'app.region': 'Region',
  'app.language': 'Language',

  // Tabs
  'nav.payment': 'Payment',
  'nav.amortization': 'Amortization',
  'nav.affordability': 'Affordability',
  'nav.refinance': 'Refinance',
  'nav.compare': 'Compare',
  'nav.report': 'Report',

  // Shared field labels
  'field.loanAmount': 'Home loan amount',
  'field.homePrice': 'Home price',
  'field.downPayment': 'Down payment',
  'field.interestRate': 'Interest rate',
  'field.loanTerm': 'Loan term',
  'field.paymentFrequency': 'Payment frequency',
  'field.years': 'years',
  'field.percent': '%',
  'field.percentPerYear': '%/yr',
  'refi.months': 'months',
  'refi.never': 'Never',
  'field.percentOfPrice': '{percent}% of price',

  'freq.monthly': 'Monthly',
  'freq.biweekly': 'Bi-weekly',
  'freq.weekly': 'Weekly',

  // Payment
  'payment.payment': 'Payment',
  'payment.totalOf': 'Total of {count} payments',
  'payment.totalInterest': 'Total interest',

  // Amortization
  'amort.extraPayment': 'Extra payment per period',
  'duration.yearsMonths': '{years} yr {months} mo',
  'duration.years': '{years} yr',
  'duration.months': '{months} mo',
  'amort.payoffDate': 'paid off {date}',
  'amort.earlier': 'earlier',
  'chart.axisEnd': 'Year {n}',
  'amort.timeSaved': 'Payoff time saved',
  'amort.interestSaved': 'Interest saved',
  'amort.newPayoff': 'New payoff',
  'amort.payments': '{count} payments',
  'amort.yearlySummary': 'Yearly summary',
  'amort.fullSchedule': 'Full schedule',
  'amort.showEvery': 'Show every payment',
  'amort.showYearly': 'Show yearly summary',
  'amort.year': 'Year',
  'amort.period': 'Period',
  'amort.paid': 'Paid',
  'amort.principal': 'Principal',
  'amort.interest': 'Interest',
  'amort.balance': 'Balance',

  // Affordability
  'aff.income': 'Gross monthly income',
  'aff.debts': 'Other monthly debts',
  'aff.downPayment': 'Down payment',
  'aff.maxDti': 'Max debt-to-income',
  'aff.propertyTaxRate': 'Property tax rate',
  'aff.insurance': 'Annual insurance',
  'aff.hoa': 'Monthly HOA',
  'aff.maxHomePrice': 'Max home price',
  'aff.maxLoan': 'Max loan amount',
  'aff.maxHousingPayment': 'Max monthly housing payment',
  'aff.frontEndDti': 'Front-end DTI',
  'aff.principalAndInterest': 'Principal & interest',
  'aff.backEndDti': 'Back-end DTI',

  // Refinance
  'refi.lifetimeSavingsNet': 'Lifetime savings (net of costs)',
  'refi.currentBalance': 'Current balance',
  'refi.currentRate': 'Current rate',
  'refi.remainingPeriods': 'Remaining payments',
  'refi.newRate': 'New rate',
  'refi.newTerm': 'New term',
  'refi.closingCosts': 'Closing costs',
  'refi.currentPayment': 'Current payment',
  'refi.newPayment': 'New payment',
  'refi.monthlySavings': 'Monthly savings',
  'refi.savingsDuringLockIn': 'during the lock-in only',
  'refi.breakEven': 'Break-even',
  'refi.lifetimeSavings': 'Lifetime savings',
  'refi.neverBreaksEven': 'Never breaks even',

  // Compare
  'cmp.addScenario': 'Add a scenario to compare.',
  'cmp.quickAdd': 'Quick add:',
  'cmp.custom': 'Custom',
  // Index names stay in Latin script in every locale: a Singapore bank's
  // own term sheet says "3M SORA".
  'preset.fixed': '{years}-Year Fixed',
  'preset.floating': 'Floating: {index} + {spread}%',
  'preset.hdbConcessionary': 'HDB concessionary',
  'rate.reverting': 'Steps up',
  'rate.initialSpread': 'Initial spread',
  'rate.lockIn': 'Lock-in',
  'rate.thereafterSpread': 'Thereafter spread',
  'rate.thenRate': 'then {rate}%',
  'rate.thenPayment': 'then {payment}',
  'preset.reverting': '{index} + {initial}% for {years} yr, then + {thereafter}%',
  'cmp.scenarioLabel': 'Scenario label',
  'cmp.customScenario': 'Custom scenario',
  'cmp.remove': 'Remove',
  'rate.fixed': 'Fixed',
  'rate.floating': 'Floating',
  'rate.rate': 'Rate',
  'cmp.term': 'Term',
  'rate.base': 'Base',
  'rate.baseFloats': 'Base rate floats',
  'note.floatingBase':
    'These figures assume the base rate stays at {base}%. It is a published benchmark that moves, this calculator does not track it, and every amount shown moves with it.',
  'rate.spread': 'Spread',
  'cmp.tradeoff':
    '{cheaper} costs {paymentDelta} more each month than {lighter}, and saves {interestDelta} in interest over the life of the loan.',
  'cmp.outright': '{label} wins on both: the lowest monthly payment and the lowest total interest.',
  'refi.termWarning':
    'This refinance runs {newTerm} against {remaining} left on your current loan — you would pay for {extra} longer. The savings below are total cash out the door, not a like-for-like comparison.',
  'cmp.scenario': 'Scenario',
  'cmp.effectiveRate': 'Rate',
  'cmp.payment': 'Payment',
  'cmp.totalPaid': 'Total paid',
  'cmp.totalInterest': 'Total interest',
  'rate.percent': '%',
  'rate.yrs': 'yrs',

  // The printable illustration. `ref.*` label the authorities the Rust
  // report cites; the codes come from mortgage-calc's `Authority`.
  'report.title': 'Mortgage illustration',
  'report.print': 'Print or save as PDF',
  'report.printNote':
    'Your browser’s print dialog makes the PDF — choose “Save as PDF” as the destination. Nothing is uploaded to produce it.',
  'report.recipients': 'Email to',
  'report.recipientsPlaceholder': 'name@example.com, another@example.com',
  'report.recipientsBad': 'Check these — they do not look like addresses: {addresses}',
  'report.email': 'Open in mail app ({count})',
  'report.emailNote':
    'Opens your own mail app with the recipients and a summary filled in — nothing is sent from this page, and nothing you type here leaves your device. A mail link cannot carry an attachment, so save the PDF above and attach it yourself.',
  'report.confirmTitle': 'Open your mail app addressed to these people?',
  'report.confirmBody':
    'Your mail app opens with a summary already written. Nothing is sent until you send it yourself, and nothing is sent from this page at any point. Remember to attach the PDF.',
  'report.confirmSend': 'Yes, open my mail app',
  'report.confirmCancel': 'Not yet',
  'report.mailSubject': 'Mortgage illustration',
  'report.mailBody':
    'Here is the illustration we discussed: {payment} a month on a {principal} loan over {years} years.',
  'report.mailSteps': 'That instalment holds for {years} yr and then rises to {payment}.',
  'report.mailAttach': 'The full document is attached as a PDF.',
  'report.subtitle': 'A worked estimate, prepared for discussion.',
  'report.watermark': 'For reference only',
  'report.prepared': 'Prepared',
  'report.market': 'Market',
  'region.US': 'United States',
  'region.SG': 'Singapore',
  'report.terms': 'Loan terms',
  'report.item': 'Item',
  'report.value': 'Value',
  'report.canChange': 'Can this change?',
  'report.no': 'No',
  'report.ratePlan': 'Yes — steps up after {years} yr, to {rate}',
  'report.andWithBenchmark': 'and whenever the benchmark moves',
  'report.paymentPlan': 'Yes — rises after {years} yr, to {payment}',
  'report.overTime': 'What you pay, over time',
  'report.period': 'Period',
  'report.instalment': 'Monthly instalment',
  'report.yearRange': 'Years {from}–{to}',
  'report.totalPaid': 'Total paid over the term',
  'report.interestShare': 'Interest as a share of everything paid',
  'report.ifRatesRise': 'If rates rise',
  'report.ifRatesRiseNote':
    'Measured against the rate this loan runs at once any lock-in has ended, since that is the rate a rise would move. Each line reprices the balance still owed at that point over the term remaining.',
  'report.increase': 'Rise',
  'report.monthlyIncrease': 'More per month',
  'report.plusPoints': '+{points}%',
  'report.schedule': 'Yearly schedule',
  'report.referenceOnly':
    'For reference only. This is an illustration produced by a calculator, not a loan offer, a quotation, or a regulated disclosure from any lender. No bank has seen these figures or agreed to them.',
  'report.sources': 'Where the rules come from',
  'report.workedAt': 'Every figure here can be reproduced, and the working inspected, at',
  'ref.MasNotice632': 'MAS Notice 632 — loan-to-value limits and the cash component of the deposit',
  'ref.MasNotice632a':
    'MAS Notice 632A — the Residential Property Loan Fact Sheet a bank must issue. This document follows its shape and is not one.',
  'ref.MasNotice645': 'MAS Notice 645 — TDSR and MSR, and the rate servicing is assessed at',
  'ref.MasSora': 'MAS — SORA, the benchmark Singapore packages are quoted over',
  'ref.Iras': "IRAS — Buyer's Stamp Duty and Additional Buyer's Stamp Duty",
  'ref.CpfBoard': 'CPF Board — Ordinary Account interest and the HDB concessionary rate',
  'ref.Cfpb':
    'CFPB — the Loan Estimate a lender must issue. This document follows its shape and is not one.',
  'ref.Fhfa': 'FHFA — the conforming loan limit',
  'ref.FederalReserveH15': 'Federal Reserve H.15 — published Prime and SOFR rates',

  // Saved scenarios
  'saved.title': 'Saved scenarios',
  'saved.empty': 'No saved scenarios yet.',
  'saved.saveAs': '+ Save current as...',
  'saved.name': 'Name',
  'saved.namePlaceholder': 'Scenario name',
  'saved.save': 'Save',
  'saved.cancel': 'Cancel',
  'saved.load': 'Load',
  'saved.delete': 'Delete',

  // Charts
  'chart.moneyGoes': 'Where your money goes',
  'chart.balanceVsInterest': 'Balance vs. interest paid',
  'chart.remainingBalance': 'Remaining balance',
  'chart.interestToDate': 'Interest paid to date',
  'chart.principalLegend': 'Principal {amount}',
  'chart.interestLegend': 'Interest {amount}',
  'chart.interestShare': 'Interest is {percent}% of everything you pay.',
  'chart.yearN': 'Year {n}',
  'chart.sharedScale':
    'Both lines share a vertical scale of {min} – {max}. They cross when interest paid so far overtakes what you still owe.',
  'chart.balanceAria':
    'Remaining balance falling from {principal} to zero, against cumulative interest reaching {interest}.',
  'chart.splitAria':
    '{principal} principal and {interest} interest, so interest is {percent} percent of everything paid.',

  // Singapore panel
  'sg.title': 'Singapore rules',
  'sg.propertyPrice': 'Property price',
  'sg.income': 'Gross monthly income',
  'sg.otherDebts': 'Other monthly debts',
  'sg.cpfAvailable': 'CPF OA available monthly',
  'sg.residency': 'Residency',
  'sg.citizen': 'Citizen',
  'sg.pr': 'Permanent Resident',
  'sg.foreigner': 'Foreigner',
  'sg.propertyCount': 'Properties owned after purchase',
  'sg.first': '1st property',
  'sg.second': '2nd property',
  'sg.thirdPlus': '3rd or more',
  'sg.propertyType': 'Property type',
  'sg.private': 'Private property',
  'sg.hdb': 'HDB flat / EC',
  'sg.loanType': 'Loan type',
  'sg.bankLoan': 'Bank loan',
  'sg.hdbLoan': 'HDB concessionary loan',
  'sg.tdsr': 'TDSR (limit 55%)',
  'sg.msr': 'MSR (limit 30%)',
  'sg.cpfUsed': 'Paid from CPF OA',
  'sg.cashMonthly': 'Cash needed monthly',
  'sg.bsd': "Buyer's Stamp Duty",
  'sg.absd': "Additional Buyer's Stamp Duty",
  'sg.downPayment': 'Down payment',
  'sg.cashAtCompletion': 'Cash needed at completion',
  'sg.assessedAt': 'Assessed at {rate}% ({instalment}/mo)',
  'sgaff.funds': 'Cash + CPF available',
  'sgaff.initialRate': 'Rate for the lock-in',
  'sgaff.thereafterRate': 'Rate thereafter',
  'sgaff.age': 'Your age',
  'sgaff.yearsOld': 'yrs',
  'sgaff.outstandingLoans': 'Housing loans outstanding',
  'sgaff.loansNone': 'None',
  'sgaff.loansOne': '1',
  'sgaff.loansTwoPlus': '2 or more',
  'sgaff.maxPrice': 'Max property price',
  'sgaff.maxLoan': 'Max loan',
  'sgaff.maxInstalment': 'Max monthly instalment',
  'sgaff.ltvNote': 'LTV limit {ltv}%',
  'sgaff.extendedTenure': 'extended tenure',
  'sgaff.minCash': 'At least {amount} must be cash, not CPF',
  'sgaff.bound.tdsr':
    'Limited by TDSR — what you can service on this income. Earning more, or clearing other debts, would raise it.',
  'sgaff.bound.msr':
    'Limited by MSR — the 30% housing-only ceiling on HDB flats and ECs, which bites before TDSR does.',
  'sgaff.bound.ltv':
    'Limited by your deposit — the loan is already at the MAS LTV ceiling, so a larger deposit would raise this, not a larger income.',
  'sg.ftaNational': 'US / Iceland / Liechtenstein / Norway / Switzerland',
  'sgaff.fixedIncome': 'Fixed monthly salary',
  'sgaff.variableIncome': 'Commission / bonus (monthly avg)',
  'sgaff.cash': 'Cash available',
  'sgaff.cpf': 'CPF OA available',
  'sgaff.cpfUsed': 'Paid from CPF OA',
  'sgaff.cashRequired': 'Cash needed at completion',
  'sgaff.cashNote':
    'CPF cannot cover the minimum cash down payment or stamp duty — both duties fall due in 14 days, before CPF can reimburse.',
  'sgaff.ftaNote':
    'Under the relevant free trade agreement you are charged ABSD at citizen rates. The remission is claimed from IRAS, not applied automatically — budget for the foreigner rate until it is granted. US nationals only (not green-card holders); Iceland, Liechtenstein, Norway and Switzerland cover nationals and PRs.',
  'sgaff.assessedIncome': 'Assessed income {amount}/mo — commission counts at 70%',

  // United States panel
  'us.title': 'US costs & PMI',
  'us.homePrice': 'Home price',
  'us.zip': 'ZIP code',
  'us.pmiRate': 'PMI rate',
  'us.useTaxDeduction': 'Estimate tax deduction',
  'us.marginalRate': 'Marginal tax rate',
  'us.yes': 'Yes',
  'us.no': 'No',
  'us.loanType': 'Loan type',
  'us.conforming': 'Conforming',
  'us.jumbo': 'Jumbo',
  'us.downPayment': 'Down payment',
  'us.propertyTax': 'Property tax',
  'us.propertyTaxWithRate': 'Property tax ({rate}%)',
  'us.pmiRequired': 'PMI (required)',
  'us.pmiNotRequired': 'PMI (not required)',
  'us.piti': 'Monthly PITI',
  'us.taxSavings': 'Tax savings',
  'us.netCost': 'Net monthly cost',
  'us.pmiHint': 'PMI applies below 20% down. Raising the down payment to {amount} removes it.',
  'us.unknownZip':
    "ZIP {zip} doesn't match a state we have a property tax rate for, so tax is excluded below.",

  // Messages produced by the Rust layer, keyed by the code it returns.
  'err.invalidPrincipal': 'Loan amount must be greater than zero (got {value}).',
  'err.invalidRate': 'Interest rate cannot be negative (got {value}).',
  'err.invalidTerm': 'Loan term must cover at least one payment (got {value}).',
  'err.termTooLong': 'Loan term is unreasonably long ({value} payments).',
  'err.downPaymentTooLarge':
    'Down payment ({downPayment}) cannot exceed the home price ({homePrice}).',
  'err.invalidIncome': 'Monthly income must be greater than zero (got {value}).',
  'err.invalidDti': 'Debt-to-income ratio must be between 0 and 1 (got {value}).',
  'err.invalidExtraPayment': 'Extra payment cannot be negative (got {value}).',
  'err.parse': 'Could not read that input: {value}',
  'err.engineUnavailable':
    'The calculator could not start. Reload the page, and if it keeps happening your browser may not support WebAssembly.',
  'err.badRequest': "Some values are missing or aren't valid numbers. Check the fields above.",
  'err.unknown': 'Something went wrong with that calculation.',

  'warn.tdsrExceeded': 'Exceeds the MAS TDSR limit of 55%.',
  'warn.msrExceeded': 'Exceeds the MAS MSR limit of 30% for HDB flats and ECs.',
  'warn.hdbLoanIneligible': 'HDB loans are only available for HDB flats and ECs bought from HDB.',
};
