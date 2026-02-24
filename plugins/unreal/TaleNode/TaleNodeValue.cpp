// TaleNode Value Type Implementation for Unreal Engine
// See TaleNodeRunner.h for FTaleValue definition.

#include "TaleNodeRunner.h"

// --- FTaleValue ---

FTaleValue FTaleValue::FromBool(bool V)
{
    FTaleValue R;
    R.Type = ETaleValueType::Bool;
    R.BoolVal = V;
    return R;
}

FTaleValue FTaleValue::FromInt(int64 V)
{
    FTaleValue R;
    R.Type = ETaleValueType::Int;
    R.IntVal = V;
    return R;
}

FTaleValue FTaleValue::FromFloat(double V)
{
    FTaleValue R;
    R.Type = ETaleValueType::Float;
    R.FloatVal = V;
    return R;
}

FTaleValue FTaleValue::FromText(const FString& V)
{
    FTaleValue R;
    R.Type = ETaleValueType::Text;
    R.TextVal = V;
    return R;
}

FTaleValue FTaleValue::FromJsonValue(const TSharedPtr<FJsonValue>& V)
{
    if (!V.IsValid() || V->IsNull())
    {
        return FromInt(0);
    }
    if (V->Type == EJson::Boolean)
    {
        return FromBool(V->AsBool());
    }
    if (V->Type == EJson::Number)
    {
        double D = V->AsNumber();
        if (D == FMath::FloorToDouble(D) && FMath::Abs(D) < 9.22e18)
        {
            return FromInt(static_cast<int64>(D));
        }
        return FromFloat(D);
    }
    if (V->Type == EJson::String)
    {
        return FromText(V->AsString());
    }
    return FromText(V->AsString());
}

bool FTaleValue::ToBool() const
{
    switch (Type)
    {
    case ETaleValueType::Bool:  return BoolVal;
    case ETaleValueType::Int:   return IntVal != 0;
    case ETaleValueType::Float: return FloatVal != 0.0;
    case ETaleValueType::Text:  return !TextVal.IsEmpty() && TextVal != TEXT("false");
    }
    return false;
}

double FTaleValue::ToDouble() const
{
    switch (Type)
    {
    case ETaleValueType::Bool:  return BoolVal ? 1.0 : 0.0;
    case ETaleValueType::Int:   return static_cast<double>(IntVal);
    case ETaleValueType::Float: return FloatVal;
    default: return 0.0;
    }
}

FString FTaleValue::ToString() const
{
    switch (Type)
    {
    case ETaleValueType::Bool:  return BoolVal ? TEXT("true") : TEXT("false");
    case ETaleValueType::Int:   return FString::Printf(TEXT("%lld"), IntVal);
    case ETaleValueType::Float: return FString::SanitizeFloat(FloatVal);
    case ETaleValueType::Text:  return TextVal;
    }
    return TEXT("");
}

bool FTaleValue::IsNumeric() const
{
    return Type == ETaleValueType::Bool
        || Type == ETaleValueType::Int
        || Type == ETaleValueType::Float;
}
