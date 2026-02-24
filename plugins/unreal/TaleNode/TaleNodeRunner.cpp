// TaleNode Runtime Dialogue Runner for Unreal Engine — Core Implementation
// Node processing is in TaleNodeProcess.cpp; FTaleValue is in TaleNodeValue.cpp.
// See TaleNodeRunner.h for API documentation.

#include "TaleNodeRunner.h"
#include "Misc/FileHelper.h"

// --- UTaleNodeRunner Public API ---

bool UTaleNodeRunner::LoadDialogue(const FString& JsonPath)
{
    FString Content;
    if (!FFileHelper::LoadFileToString(Content, *JsonPath))
    {
        UE_LOG(LogTemp, Warning, TEXT("[TaleNodeRunner] Cannot open file: %s"), *JsonPath);
        return false;
    }
    return LoadFromString(Content);
}

bool UTaleNodeRunner::LoadFromString(const FString& JsonString)
{
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    TSharedPtr<FJsonObject> Parsed;
    if (!FJsonSerializer::Deserialize(Reader, Parsed) || !Parsed.IsValid())
    {
        UE_LOG(LogTemp, Warning, TEXT("[TaleNodeRunner] Invalid JSON"));
        return false;
    }

    Data = Parsed;
    NodeMap.Empty();
    Variables.Empty();
    Characters.Empty();
    CurrentNodeId = TEXT("");
    bRunning = false;
    CurrentFilteredOptions.Empty();

    // Build node map
    const TArray<TSharedPtr<FJsonValue>>* NodesArray;
    if (Data->TryGetArrayField(TEXT("nodes"), NodesArray))
    {
        for (const auto& Elem : *NodesArray)
        {
            TSharedPtr<FJsonObject> NodeObj = Elem->AsObject();
            if (NodeObj.IsValid())
            {
                FString Id;
                if (NodeObj->TryGetStringField(TEXT("id"), Id))
                    NodeMap.Add(Id, NodeObj);
            }
        }
    }

    // Initialize variables
    const TArray<TSharedPtr<FJsonValue>>* VarsArray;
    if (Data->TryGetArrayField(TEXT("variables"), VarsArray))
    {
        for (const auto& Elem : *VarsArray)
        {
            TSharedPtr<FJsonObject> VarObj = Elem->AsObject();
            if (VarObj.IsValid())
            {
                FString Name;
                if (VarObj->TryGetStringField(TEXT("name"), Name))
                {
                    Variables.Add(Name, FTaleValue::FromJsonValue(
                        VarObj->TryGetField(TEXT("default"))));
                }
            }
        }
    }

    // Build character map
    const TArray<TSharedPtr<FJsonValue>>* CharsArray;
    if (Data->TryGetArrayField(TEXT("characters"), CharsArray))
    {
        for (const auto& Elem : *CharsArray)
        {
            TSharedPtr<FJsonObject> CharObj = Elem->AsObject();
            if (CharObj.IsValid())
            {
                FString Id;
                if (CharObj->TryGetStringField(TEXT("id"), Id))
                    Characters.Add(Id, CharObj);
            }
        }
    }

    return true;
}

void UTaleNodeRunner::Start(const FString& StartNodeId)
{
    if (NodeMap.Num() == 0)
    {
        UE_LOG(LogTemp, Warning, TEXT("[TaleNodeRunner] No dialogue loaded"));
        return;
    }

    if (!StartNodeId.IsEmpty())
    {
        CurrentNodeId = StartNodeId;
    }
    else
    {
        CurrentNodeId = TEXT("");
        for (const auto& Pair : NodeMap)
        {
            FString NodeType;
            if (Pair.Value->TryGetStringField(TEXT("type"), NodeType)
                && NodeType == TEXT("start"))
            {
                CurrentNodeId = Pair.Key;
                break;
            }
        }
        if (CurrentNodeId.IsEmpty())
        {
            UE_LOG(LogTemp, Warning, TEXT("[TaleNodeRunner] No start node found"));
            return;
        }
    }

    bRunning = true;
    OnDialogueStarted.Broadcast();
    ProcessNode();
}

void UTaleNodeRunner::Advance()
{
    if (!bRunning) return;
    auto Node = GetCurrentNode();
    if (!Node.IsValid()) { End(TEXT("")); return; }
    FString NextId;
    Node->TryGetStringField(TEXT("next"), NextId);
    GoTo(NextId);
}

void UTaleNodeRunner::Choose(int32 OptionIndex)
{
    if (!bRunning) return;
    auto Node = GetCurrentNode();
    if (!Node.IsValid()) return;
    FString NodeType;
    Node->TryGetStringField(TEXT("type"), NodeType);
    if (NodeType != TEXT("choice")) return;

    if (OptionIndex < 0 || OptionIndex >= CurrentFilteredOptions.Num())
    {
        UE_LOG(LogTemp, Warning,
            TEXT("[TaleNodeRunner] Invalid option index: %d"), OptionIndex);
        return;
    }

    FString NextId;
    CurrentFilteredOptions[OptionIndex]->TryGetStringField(TEXT("next"), NextId);
    GoTo(NextId);
}

FTaleValue UTaleNodeRunner::GetVariable(const FString& Name) const
{
    if (const FTaleValue* Val = Variables.Find(Name))
        return *Val;
    return FTaleValue::FromInt(0);
}

void UTaleNodeRunner::SetVariable(const FString& Name, const FTaleValue& Value)
{
    Variables.Add(Name, Value);
    OnVariableChanged.Broadcast(Name, Value.ToString());
}

void UTaleNodeRunner::Stop()
{
    bRunning = false;
    CurrentNodeId = TEXT("");
    CurrentFilteredOptions.Empty();
}

// --- Internal navigation ---

TSharedPtr<FJsonObject> UTaleNodeRunner::GetCurrentNode() const
{
    if (CurrentNodeId.IsEmpty()) return nullptr;
    const auto* Found = NodeMap.Find(CurrentNodeId);
    return Found ? *Found : nullptr;
}

void UTaleNodeRunner::GoTo(const FString& NextId)
{
    if (NextId.IsEmpty()) { End(TEXT("")); return; }
    CurrentNodeId = NextId;
    ProcessNode();
}

void UTaleNodeRunner::End(const FString& Tag)
{
    bRunning = false;
    CurrentNodeId = TEXT("");
    CurrentFilteredOptions.Empty();
    OnDialogueEnded.Broadcast(Tag);
}

// --- Condition evaluation ---

bool UTaleNodeRunner::EvaluateCondition(const TSharedPtr<FJsonObject>& Cond) const
{
    FString VarName, Op;
    Cond->TryGetStringField(TEXT("variable"), VarName);
    Cond->TryGetStringField(TEXT("operator"), Op);
    if (Op.IsEmpty()) Op = TEXT("==");
    return EvaluateConditionFields(VarName, Op, Cond->TryGetField(TEXT("value")));
}

bool UTaleNodeRunner::EvaluateConditionFields(
    const FString& VarName,
    const FString& Op,
    const TSharedPtr<FJsonValue>& Value) const
{
    const FTaleValue* CurrentPtr = Variables.Find(VarName);
    if (!CurrentPtr) return false;
    FTaleValue Current = *CurrentPtr;
    FTaleValue Target = FTaleValue::FromJsonValue(Value);

    if (Op == TEXT("=="))       return LooseEqual(Current, Target);
    if (Op == TEXT("!="))       return !LooseEqual(Current, Target);
    if (Op == TEXT(">"))        return Current.ToDouble() > Target.ToDouble();
    if (Op == TEXT("<"))        return Current.ToDouble() < Target.ToDouble();
    if (Op == TEXT(">="))       return Current.ToDouble() >= Target.ToDouble();
    if (Op == TEXT("<="))       return Current.ToDouble() <= Target.ToDouble();
    if (Op == TEXT("contains")) return Current.ToString().Contains(Target.ToString());
    return false;
}

bool UTaleNodeRunner::LooseEqual(const FTaleValue& A, const FTaleValue& B)
{
    if (A.Type == ETaleValueType::Bool && B.Type == ETaleValueType::Bool)
        return A.BoolVal == B.BoolVal;
    if (A.Type == ETaleValueType::Text && B.Type == ETaleValueType::Text)
        return A.TextVal == B.TextVal;
    if (A.IsNumeric() && B.IsNumeric())
        return FMath::Abs(A.ToDouble() - B.ToDouble()) < 1e-9;
    return A.ToString() == B.ToString();
}

// --- Text interpolation ---

FString UTaleNodeRunner::InterpolateText(
    const FString& Text,
    const TMap<FString, FTaleValue>& Vars)
{
    if (!Text.Contains(TEXT("{")))
        return Text;

    FString Result;
    int32 Len = Text.Len();
    int32 I = 0;

    while (I < Len)
    {
        if (Text[I] == '{')
        {
            I++;
            int32 Start = I;
            int32 Depth = 1;
            while (I < Len && Depth > 0)
            {
                if (Text[I] == '{') Depth++;
                else if (Text[I] == '}') Depth--;
                if (Depth > 0) I++;
            }
            FString VarName = Text.Mid(Start, I - Start).TrimStartAndEnd();
            if (I < Len) I++;

            if (const FTaleValue* Val = Vars.Find(VarName))
                Result += Val->ToString();
            else
                Result += TEXT("{") + VarName + TEXT("}");
        }
        else
        {
            Result += Text[I];
            I++;
        }
    }
    return Result;
}
