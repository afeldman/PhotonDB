# Term types - represents query operations
@0xc3d4e5f6a7b8c9da;

using Types = import "types.capnp";

# Term types (RethinkDB operations)
enum TermType {
    # Core
    datum @0;
    makeArray @1;
    makeObj @2;
    
    # Variables and functions
    var @3;
    javascript @4;
    uuid @5;
    http @6;
    error @7;
    implicitVar @8;
    
    # Database operations
    db @9;
    table @10;
    get @11;
    getAll @12;
    
    # Comparisons
    eq @13;
    ne @14;
    lt @15;
    le @16;
    gt @17;
    ge @18;
    not @19;
    
    # Arithmetic
    add @20;
    sub @21;
    mul @22;
    div @23;
    mod @24;
    floor @25;
    ceil @26;
    round @27;
    
    # Array operations
    append @28;
    prepend @29;
    difference @30;
    
    # Set operations
    setInsert @31;
    setIntersection @32;
    setUnion @33;
    setDifference @34;
    
    # Sequence operations
    slice @35;
    skip @36;
    limit @37;
    offsetsOf @38;
    contains @39;
    
    # Object operations
    getField @40;
    keys @41;
    values @42;
    object @43;
    hasFields @44;
    withFields @45;
    pluck @46;
    without @47;
    merge @48;
    
    # Sequence transformations
    between @49;
    reduce @50;
    map @51;
    fold @52;
    filter @53;
    concatMap @54;
    orderBy @55;
    distinct @56;
    count @57;
    isEmpty @58;
    union @59;
    nth @60;
    bracket @61;
    
    # Joins
    innerJoin @62;
    outerJoin @63;
    eqJoin @64;
    zip @65;
    range @66;
    
    # Array operations (indexed)
    insertAt @67;
    deleteAt @68;
    changeAt @69;
    spliceAt @70;
    
    # Type operations
    coerceTo @71;
    typeOf @72;
    
    # Write operations
    update @73;
    delete @74;
    replace @75;
    insert @76;
    
    # Admin operations
    dbCreate @77;
    dbDrop @78;
    dbList @79;
    tableCreate @80;
    tableDrop @81;
    tableList @82;
    config @83;
    status @84;
    wait @85;
    reconfigure @86;
    rebalance @87;
    sync @88;
    grant @89;
    
    # Index operations
    indexCreate @90;
    indexDrop @91;
    indexList @92;
    indexStatus @93;
    indexWait @94;
    indexRename @95;
    
    # Write hooks
    setWriteHook @96;
    getWriteHook @97;
    
    # Control flow
    funcall @98;
    branch @99;
    or @100;
    and @101;
    forEach @102;
    func @103;
    
    # Ordering
    asc @104;
    desc @105;
    
    # Utility
    info @106;
    match @107;
    upcase @108;
    downcase @109;
    sample @110;
    default @111;
    
    # JSON
    json @112;
    
    # Time operations
    iso8601 @113;
    toIso8601 @114;
    epochTime @115;
    toEpochTime @116;
    now @117;
    inTimezone @118;
    during @119;
    date @120;
    timeOfDay @121;
    timezone @122;
    year @123;
    month @124;
    day @125;
    dayOfWeek @126;
    dayOfYear @127;
    hours @128;
    minutes @129;
    seconds @130;
    time @131;
    
    # Time constants
    monday @132;
    tuesday @133;
    wednesday @134;
    thursday @135;
    friday @136;
    saturday @137;
    sunday @138;
    january @139;
    february @140;
    march @141;
    april @142;
    may @143;
    june @144;
    july @145;
    august @146;
    september @147;
    october @148;
    november @149;
    december @150;
    
    # Aggregation
    literal @151;
    group @152;
    sum @153;
    avg @154;
    min @155;
    max @156;
    ungroup @157;
    
    # String operations
    split @158;
    
    # Random
    random @159;
    
    # Changefeeds
    changes @160;
    args @161;
    
    # Binary
    binary @162;
    
    # Geometry
    geojson @163;
    toGeojson @164;
    point @165;
    line @166;
    polygon @167;
    distance @168;
    intersects @169;
    includes @170;
    circle @171;
    getIntersecting @172;
    fill @173;
    getNearest @174;
    polygonSub @175;
    
    # Conversion
    toJsonString @176;
    
    # Constants
    minval @177;
    maxval @178;
    
    # Bitwise operations
    bitAnd @179;
    bitOr @180;
    bitXor @181;
    bitNot @182;
    bitSal @183;
    bitSar @184;
    
    # Vector operations (AI/ML)
    indexCreateVector @185;
    vector @186;
    vectorDistance @187;
    getNearestVector @188;
    getInRangeVector @189;
    
    # Calculus operations (Scientific Computing)
    # Differentiation
    derivative @190;           # First derivative: dy/dx
    derivative2 @191;          # Second derivative: d²y/dx²
    partialDerivative @192;    # Partial derivative: ∂f/∂x
    gradient @193;             # Gradient: ∇f
    divergence @194;           # Divergence: ∇·F
    curl @195;                 # Curl: ∇×F
    laplacian @196;            # Laplacian: ∇²f
    
    # Integration
    integrate @197;            # Definite integral
    cumulativeIntegrate @198;  # Cumulative integral (running sum)
    integrateVolume @199;      # Multi-dimensional integral
    
    # Differential equations
    solveOde @200;             # Solve ODE: dy/dt = f(t,y)
    solveOde2 @201;            # Second-order ODE
    solveOdeSystem @202;       # System of ODEs
    solvePdeHeat @203;         # Heat equation (PDE)
    solvePdeWave @204;         # Wave equation (PDE)
    solvePdePoisson @205;      # Poisson equation (PDE)
    solveNBody @206;           # N-body gravitational simulation
    
    # Signal processing
    fft @207;                  # Fast Fourier Transform
    ifft @208;                 # Inverse FFT
    psd @209;                  # Power spectral density
    convolve @210;             # Convolution
    correlate @211;            # Cross-correlation
    hilbertTransform @212;     # Hilbert transform
    
    # Numerical methods
    interpolate @213;          # Interpolation (linear, cubic, spline)
    extrapolate @214;          # Extrapolation
    smooth @215;               # Smoothing (moving average, Savitzky-Golay)
    resample @216;             # Resampling
    findPeaks @217;            # Peak detection
    findZeros @218;            # Root finding
    
    # Statistical Analysis (Polars-like)
    # Descriptive statistics
    stats @219;                # Descriptive statistics (mean, median, std, etc.)
    describe @220;             # Full statistical summary
    quantile @221;             # Quantiles/percentiles
    skewness @222;             # Skewness (asymmetry)
    kurtosis @223;             # Kurtosis (tail weight)
    
    # Aggregations
    variance @224;             # Variance
    stdDev @225;               # Standard deviation
    median @226;               # Median
    mode @227;                 # Mode
    iqr @228;                  # Interquartile range
    mad @229;                  # Median absolute deviation
    
    # Rolling/Expanding windows
    rollingMean @230;          # Rolling average
    rollingStd @231;           # Rolling standard deviation
    rollingQuantile @232;      # Rolling quantile
    expandingMean @233;        # Expanding mean (cumulative)
    expandingSum @234;         # Expanding sum
    
    # Correlation and covariance
    corr @235;                 # Correlation matrix
    corrPairwise @236;         # Pairwise correlation
    cov @237;                  # Covariance matrix
    partialCorr @238;          # Partial correlation
    
    # Hypothesis testing
    tTest @239;                # Student's t-test
    pairedTTest @240;          # Paired t-test
    anova @241;                # Analysis of variance
    chiSquareTest @242;        # Chi-square test
    kolmogorovSmirnov @243;    # K-S test
    shapiroWilk @244;          # Test for normality
    mannWhitneyU @245;         # Mann-Whitney U test
    kruskalWallis @246;        # Kruskal-Wallis test
    
    # Regression
    linearRegression @247;     # Linear regression
    polynomialRegression @248; # Polynomial regression
    logisticRegression @249;   # Logistic regression
    ridgeRegression @250;      # Ridge (L2) regression
    lassoRegression @251;      # Lasso (L1) regression
    
    # Time series statistics
    autocorr @252;             # Autocorrelation
    pacf @253;                 # Partial autocorrelation
    detectSeasonality @254;    # Seasonality detection
    decompose @255;            # Time series decomposition (STL)
    arima @256;                # ARIMA modeling
    exponentialSmoothing @257; # Exponential smoothing
    
    # Distributions
    distribution @258;         # Statistical distributions
    fitDistribution @259;      # Fit distribution to data
    
    # Data preprocessing
    standardize @260;          # Z-score normalization
    normalize @261;            # Min-max normalization
    robustScale @262;          # Robust scaling (median/IQR)
    oneHotEncode @263;         # One-hot encoding
    labelEncode @264;          # Label encoding
    bin @265;                  # Binning/discretization
    
    # Business analytics
    cohortAnalysis @266;       # Cohort analysis
    funnelAnalysis @267;       # Funnel conversion
    rfmAnalysis @268;          # RFM analysis
    abTestPower @269;          # A/B test power calculation
    survivalAnalysis @270;     # Survival analysis (Kaplan-Meier)
    
    # Outlier detection
    detectOutliers @271;       # Outlier detection (zscore, IQR)
    detectOutliersMl @272;     # ML-based outlier detection
}

# A Term is either a piece of data or an operator with operands
struct Term {
    type @0 :TermType;
    
    # For DATUM type
    datum @1 :Types.Datum;
    
    # Positional arguments
    args @2 :List(Term);
    
    # Optional arguments (named parameters)
    struct OptArg {
        key @0 :Text;
        value @1 :Term;
    }
    optargs @3 :List(OptArg);
}
