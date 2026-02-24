// TaleNode Node Processing — per-type node handlers.
// Split from TaleNodeRunner.cpp for maintainability.

#include "TaleNodeRunner.h"

void UTaleNodeRunner::ProcessNode()
{
    if (!bRunning) return;
    auto Node = GetCurrentNode();
    if (!Node.IsValid()) { End(TEXT("")); return; }

    FString NodeType;
    Node->TryGetStringField(TEXT("type"), NodeType);

    if      (NodeType == TEXT("start"))     { FString N; Node->TryGetStringField(TEXT("next"), N); GoTo(N); }
    else if (NodeType == TEXT("dialogue"))  { ProcessDialogue(Node); }
    else if (NodeType == TEXT("choice"))    { ProcessChoice(Node); }
    else if (NodeType == TEXT("condition")) { ProcessCondition(Node); }
    else if (NodeType == TEXT("event"))     { ProcessEvent(Node); }
    else if (NodeType == TEXT("random"))    { ProcessRandom(Node); }
    else if (NodeType == TEXT("end"))       { FString T; Node->TryGetStringField(TEXT("tag"), T); End(T); }
    else { UE_LOG(LogTemp, Warning, TEXT("[TaleNodeRunner] Unknown type: %s"), *NodeType); End(TEXT("")); }
}

void UTaleNodeRunner::ProcessDialogue(const TSharedPtr<FJsonObject>& Node)
{
    FString SpeakerId; Node->TryGetStringField(TEXT("speaker"), SpeakerId);
    FString SpeakerName = SpeakerId;
    if (auto* CharObj = Characters.Find(SpeakerId))
    {
        FString CharName;
        if ((*CharObj)->TryGetStringField(TEXT("name"), CharName))
            SpeakerName = CharName;
    }

    FString RawText; Node->TryGetStringField(TEXT("text"), RawText);
    FString Text = InterpolateText(RawText, Variables);
    FString Emotion, Portrait, Audio, NodeId;
    Node->TryGetStringField(TEXT("emotion"), Emotion);
    Node->TryGetStringField(TEXT("portrait"), Portrait);
    Node->TryGetStringField(TEXT("audio"), Audio);
    Node->TryGetStringField(TEXT("id"), NodeId);

    OnDialogueLine.Broadcast(SpeakerName, Text, Emotion, Portrait, Audio, NodeId);
}

void UTaleNodeRunner::ProcessChoice(const TSharedPtr<FJsonObject>& Node)
{
    FString Prompt; Node->TryGetStringField(TEXT("prompt"), Prompt);
    Prompt = InterpolateText(Prompt, Variables);

    CurrentFilteredOptions.Empty();
    TArray<FString> OptionTexts;

    const TArray<TSharedPtr<FJsonValue>>* OptionsArray;
    if (Node->TryGetArrayField(TEXT("options"), OptionsArray))
    {
        for (const auto& Elem : *OptionsArray)
        {
            auto OptObj = Elem->AsObject();
            if (!OptObj.IsValid()) continue;
            const TSharedPtr<FJsonObject>* CondPtr;
            if (OptObj->TryGetObjectField(TEXT("condition"), CondPtr) && CondPtr->IsValid())
            {
                if (!EvaluateCondition(*CondPtr)) continue;
            }
            CurrentFilteredOptions.Add(OptObj);
            FString RawText; OptObj->TryGetStringField(TEXT("text"), RawText);
            OptionTexts.Add(InterpolateText(RawText, Variables));
        }
    }
    OnChoicePresented.Broadcast(Prompt, OptionTexts);
}

void UTaleNodeRunner::ProcessCondition(const TSharedPtr<FJsonObject>& Node)
{
    FString VarName, Op;
    Node->TryGetStringField(TEXT("variable"), VarName);
    Node->TryGetStringField(TEXT("operator"), Op);
    if (Op.IsEmpty()) Op = TEXT("==");

    bool Result = EvaluateConditionFields(VarName, Op, Node->TryGetField(TEXT("value")));
    FString NextId;
    if (Result) Node->TryGetStringField(TEXT("true_next"), NextId);
    else        Node->TryGetStringField(TEXT("false_next"), NextId);
    GoTo(NextId);
}

void UTaleNodeRunner::ProcessEvent(const TSharedPtr<FJsonObject>& Node)
{
    const TArray<TSharedPtr<FJsonValue>>* ActionsArray;
    if (Node->TryGetArrayField(TEXT("actions"), ActionsArray))
    {
        for (const auto& Elem : *ActionsArray)
        {
            auto ActionObj = Elem->AsObject();
            if (!ActionObj.IsValid()) continue;
            FString ActionType, Key;
            ActionObj->TryGetStringField(TEXT("action"), ActionType);
            ActionObj->TryGetStringField(TEXT("key"), Key);
            auto ValueField = ActionObj->TryGetField(TEXT("value"));

            FString Normalized = ActionType.ToLower().Replace(TEXT("_"), TEXT(""));
            if (Normalized == TEXT("setvariable"))
            {
                FTaleValue TV = FTaleValue::FromJsonValue(ValueField);
                Variables.Add(Key, TV);
                OnVariableChanged.Broadcast(Key, TV.ToString());
            }
            else
            {
                FString ValueStr = ValueField.IsValid() ? ValueField->AsString() : TEXT("");
                OnEventTriggered.Broadcast(ActionType, Key, ValueStr);
            }
        }
    }
    FString NextId; Node->TryGetStringField(TEXT("next"), NextId);
    GoTo(NextId);
}

void UTaleNodeRunner::ProcessRandom(const TSharedPtr<FJsonObject>& Node)
{
    const TArray<TSharedPtr<FJsonValue>>* BranchesArray;
    if (!Node->TryGetArrayField(TEXT("branches"), BranchesArray)
        || BranchesArray->Num() == 0)
    {
        FString NextId; Node->TryGetStringField(TEXT("next"), NextId);
        GoTo(NextId);
        return;
    }

    double TotalWeight = 0.0;
    for (const auto& Elem : *BranchesArray)
    {
        auto BranchObj = Elem->AsObject();
        if (BranchObj.IsValid())
            TotalWeight += BranchObj->GetNumberField(TEXT("weight"));
    }

    if (TotalWeight <= 0.0)
    {
        int32 Idx = FMath::RandRange(0, BranchesArray->Num() - 1);
        auto Chosen = (*BranchesArray)[Idx]->AsObject();
        FString NextId;
        if (Chosen.IsValid()) Chosen->TryGetStringField(TEXT("next"), NextId);
        GoTo(NextId);
        return;
    }

    double Roll = FMath::FRand() * TotalWeight;
    double Cumulative = 0.0;
    for (const auto& Elem : *BranchesArray)
    {
        auto BranchObj = Elem->AsObject();
        if (!BranchObj.IsValid()) continue;
        Cumulative += BranchObj->GetNumberField(TEXT("weight"));
        if (Roll <= Cumulative)
        {
            FString NextId; BranchObj->TryGetStringField(TEXT("next"), NextId);
            GoTo(NextId);
            return;
        }
    }
    auto Last = BranchesArray->Last()->AsObject();
    FString NextId;
    if (Last.IsValid()) Last->TryGetStringField(TEXT("next"), NextId);
    GoTo(NextId);
}
